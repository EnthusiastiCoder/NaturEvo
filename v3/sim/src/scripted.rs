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
