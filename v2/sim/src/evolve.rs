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
