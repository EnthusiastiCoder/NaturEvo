//! Style evaluation: pair fighters (teachers or trained champions), run fights over fixed seeds and report win
//! rates and behavioural metrics per style. Measurement only; nothing here feeds back into fitness.
use crate::arena::Arena;
use crate::duel::{Brain, Match, MatchResult, Outcome};
use crate::grace::match_grace;
use crate::moveset::N_MOVES;
use crate::net::N_OUT;

/// Behaviour totals of one side across fights (divide by `n` for means).
#[derive(Clone, Debug, Default)]
pub struct SideAcc {
    pub n: u32,
    pub wins: u32,
    pub draws: u32,
    pub capped: u32,
    pub ticks: f64,
    pub distinct: f64,
    pub union_moves: [bool; N_MOVES],
    pub gap_mean: f64,
    pub engaged_gap: f64,
    pub spacing_std: f64,
    pub hits_per_swing: f64,
    pub whiff: f64,
    pub punish: f64,
    pub parry_hits: f64,
    pub defences: f64,
    pub parry_defs: f64,
    pub combo_mean: f64,
    pub combo_max: f64,
    pub jerk: f64,
    pub tip_jerk: f64,
    pub swings: f64,
    pub hits: f64,
    pub usage: [f64; N_MOVES],
}

impl SideAcc {
    pub fn add(&mut self, m: &Match, r: &MatchResult, who: usize) {
        let g = match_grace(m, who);
        self.n += 1;
        let mine = if who == 0 { Outcome::WinA } else { Outcome::WinB };
        if r.outcome == mine {
            self.wins += 1;
        }
        if r.outcome == Outcome::Draw {
            self.draws += 1;
        }
        self.capped += r.capped as u32;
        self.ticks += r.ticks as f64;
        self.distinct += g.usage.iter().filter(|u| **u > 0.0).count() as f64;
        for i in 0..N_MOVES {
            if g.usage[i] > 0.0 {
                self.union_moves[i] = true;
            }
            self.usage[i] += g.usage[i] as f64;
        }
        self.gap_mean += r.gap_mean as f64;
        self.engaged_gap += r.engaged_gap as f64;
        self.spacing_std += g.spacing_std as f64;
        self.hits_per_swing += g.hits_per_swing as f64;
        self.whiff += g.whiff_rate as f64;
        self.punish += g.punish_rate as f64;
        let d = &r.defence[who];
        let tot = (d.parry + d.block + d.dodge) as f64;
        self.defences += tot;
        self.parry_defs += d.parry as f64;
        self.combo_mean += r.combo_mean[who] as f64;
        self.combo_max += r.combo_max[who] as f64;
        self.jerk += r.jerk[who] as f64;
        self.tip_jerk += g.jerk as f64;
        self.swings += r.swings[who] as f64;
        self.hits += r.hits[who] as f64;
    }
    fn mean(&self, v: f64) -> f64 {
        v / self.n.max(1) as f64
    }
    pub fn win_rate(&self) -> f64 {
        self.mean(self.wins as f64)
    }
    pub fn distinct_moves(&self) -> f64 {
        self.mean(self.distinct)
    }
    pub fn gap(&self) -> f64 {
        self.mean(self.gap_mean)
    }
    pub fn hits_per_swing(&self) -> f64 {
        self.mean(self.hits_per_swing)
    }
    pub fn capped_share(&self) -> f64 {
        self.mean(self.capped as f64)
    }
    pub fn mean_ticks(&self) -> f64 {
