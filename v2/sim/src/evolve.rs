//! Co-evolution of hero and monster populations. Deterministic for a given seed
//! regardless of thread count: every match is a pure function of its inputs and
//! results are collected by index.
use crate::config::ArenaConfig;
use crate::fighter::Role;
use crate::genome::Genome;
use crate::matchup::{play, Match, MatchResult, Outcome};
use crate::rng::Rng;
use crate::world::Arena;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub struct TrainConfig {
    pub seed: u64,
    pub generations: u32,
    pub pop: usize,
    pub k_opponents: usize,
    pub hof_sample: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActionFreq {
    pub attack: f32,
    pub block: f32,
    pub dodge: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenStats {
    pub gen: u32,
    pub best_fitness: [f32; 2],
    pub mean_fitness: [f32; 2],
    pub win_rate_vs_hof: [f32; 2],
    pub win_rate_vs_random: [f32; 2],
    pub action_freq: [ActionFreq; 2],
    pub avg_ticks: f32,
}

impl GenStats {
    pub fn to_json(&self) -> String {
        let r = ["hero", "monster"];
        let mut s = format!("{{\"gen\":{}", self.gen);
        let _ = write!(s, ",\"avg_ticks\":{}", self.avg_ticks);
        for (key, v) in [
            ("best_fitness", &self.best_fitness),
            ("mean_fitness", &self.mean_fitness),
            ("win_rate_vs_hof", &self.win_rate_vs_hof),
            ("win_rate_vs_random", &self.win_rate_vs_random),
        ] {
            let _ = write!(s, ",\"{}\":{{\"hero\":{},\"monster\":{}}}", key, v[0], v[1]);
        }
        let _ = write!(s, ",\"action_freq\":{{");
        for i in 0..2 {
            let a = &self.action_freq[i];
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "\"{}\":{{\"attack\":{},\"block\":{},\"dodge\":{}}}", r[i], a.attack, a.block, a.dodge);
        }
        s.push_str("}}");
        s
    }

    pub fn csv_header() -> &'static str {
        "gen,avg_ticks,best_hero,best_monster,mean_hero,mean_monster,hof_wr_hero,hof_wr_monster,rnd_wr_hero,rnd_wr_monster,hero_attack,hero_block,hero_dodge,monster_attack,monster_block,monster_dodge"
    }

    pub fn to_csv(&self) -> String {
        let a = &self.action_freq;
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.gen, self.avg_ticks, self.best_fitness[0], self.best_fitness[1], self.mean_fitness[0], self.mean_fitness[1],
            self.win_rate_vs_hof[0], self.win_rate_vs_hof[1], self.win_rate_vs_random[0], self.win_rate_vs_random[1],
            a[0].attack, a[0].block, a[0].dodge, a[1].attack, a[1].block, a[1].dodge
        )
    }

    fn to_nums(&self) -> String {
        let a = &self.action_freq;
        let v: Vec<String> = [
            self.gen as f32, self.avg_ticks, self.best_fitness[0], self.best_fitness[1], self.mean_fitness[0], self.mean_fitness[1],
            self.win_rate_vs_hof[0], self.win_rate_vs_hof[1], self.win_rate_vs_random[0], self.win_rate_vs_random[1],
            a[0].attack, a[0].block, a[0].dodge, a[1].attack, a[1].block, a[1].dodge,
        ]
        .iter()
        .map(|x| format!("{}", x))
        .collect();
        v.join(" ")
    }

    fn from_nums(s: &str) -> Result<GenStats, String> {
        let v: Vec<f32> = s.split_whitespace().map(|x| x.parse::<f32>().map_err(|e| e.to_string())).collect::<Result<_, _>>()?;
        if v.len() != 16 {
            return Err("bad stats line".into());
        }
        let af = |i: usize| ActionFreq { attack: v[i], block: v[i + 1], dodge: v[i + 2] };
        Ok(GenStats {
            gen: v[0] as u32,
            avg_ticks: v[1],
            best_fitness: [v[2], v[3]],
            mean_fitness: [v[4], v[5]],
            win_rate_vs_hof: [v[6], v[7]],
            win_rate_vs_random: [v[8], v[9]],
            action_freq: [af(10), af(13)],
        })
    }
}

const HOF_CAP: usize = 64;
const ELITES: usize = 2;
const BASELINE_N: usize = 10;

/// Per-match outcome for the evaluated role.
#[derive(Clone, Copy, Debug)]
struct MatchScore {
    fitness: f32,
    win: bool,
    ticks: u32,
    freq: [f32; 3],
}
