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

const WINDUP: f32 = 0.3;
const STRIKE: f32 = 0.2;
const RECOVER: f32 = 0.35;
/// Preferred torso distance for swinging, metres.
const REACH: f32 = 2.4;
/// Ticks without progress after which the attacker swings from where it is (an obstacle is in the way).
const STUCK_S: f32 = 0.5;

fn set(t: &mut [f32; 12], j: Joint, v: f32) {
    t[j as usize] = v;
}

fn lerp(a: f32, b: f32, s: f32) -> f32 {
    a + (b - a) * s.clamp(0.0, 1.0)
}
