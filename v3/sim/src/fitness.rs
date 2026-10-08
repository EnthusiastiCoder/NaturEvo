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
