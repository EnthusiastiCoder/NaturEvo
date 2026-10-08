//! Fitness. There is no block bit and no guard bonus: defence pays only through per-threat credit.
use crate::config::{DRAW_SCORE, IMPACT_REF};
use crate::duel::{MatchResult, Outcome, Resolution};
use crate::robot::Part;

pub const W_WIN: f32 = 4.0;
pub const W_DEALT: f32 = 3.0;
pub const W_TAKEN: f32 = 2.5;
pub const W_DEFENCE: f32 = 2.0;
pub const W_IDLE_GUARD: f32 = 1.0;
pub const W_PASSIVITY: f32 = 1.0;
pub const W_CHAIN: f32 = 0.5;
pub const KO_BONUS: f32 = 1.0;
pub const MAX_CREDITED_THREATS: usize = 6;

#[derive(Clone, Copy, Debug, Default)]
pub struct MatchStats {
    pub win: bool,
    pub draw: bool,
    pub ko: bool,
    pub dmg_dealt_norm: f32,
    pub dmg_taken_norm: f32,
    pub defence_credit: f32,
    pub idle_guard: f32,
    pub passivity: f32,
    pub chain_quality: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThreatOutcome {
    pub resolution: Resolution,
    pub i_potential: f32,
    pub followup_hit: bool,
}

/// Zone weight of a part in damage dealt: head 1.5, core 1.0, limbs 0.7.
pub fn zone_weight(p: Part) -> f32 {
    match p {
        Part::Head => 1.5,
        Part::Torso | Part::Pelvis => 1.0,
        _ => 0.7,
    }
}
