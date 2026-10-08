//! Scripted opponents: a static dummy and an attacker that approaches and swings at random
//! intervals (slash high, slash low, thrust). Deterministic from the match seed.
use crate::robot::{Joint, Robot, STAND_TARGETS};
use crate::rng::Rng;

/// Normalised joint targets plus a drive command.
#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub targets: [f32; 12],
    pub drive: f32,
}

pub fn dummy() -> Action {
    Action { targets: STAND_TARGETS, drive: 0.0 }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Swing {
    SlashHigh,
    SlashLow,
    Thrust,
}

#[derive(Clone, Debug)]
pub struct Scripted {
    rng: Rng,
    swing: Option<(Swing, f32)>,
    wait: f32,
    last_x: f32,
    still: f32,
}
