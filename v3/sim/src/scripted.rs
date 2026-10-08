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

impl Scripted {
    pub fn new(seed: u64) -> Scripted {
        let mut rng = Rng::new(seed ^ 0x5C21_97ED);
        let wait = rng.range(0.4, 1.2);
        Scripted { rng, swing: None, wait, last_x: f32::NAN, still: 0.0 }
    }

    /// Joint targets for one swing at time `t` since it began: (shoulder, elbow, waist, hip) of the sword arm.
    fn pose(kind: Swing, t: f32) -> (f32, f32, f32, f32) {
        let (w, s) = match kind {
            Swing::SlashHigh => ((0.85, 0.2, -0.5, -0.1), (0.05, -0.5, 0.5, 0.0)),
            Swing::SlashLow => ((0.5, 0.1, -0.5, -0.1), (-0.5, -0.6, 0.6, 0.0)),
            Swing::Thrust => ((0.0, 0.8, -0.3, -0.1), (0.2, -0.9, 0.4, 0.0)),
        };
        let rest = (-0.4, 0.0, 0.0, -0.2);
        let lerp4 = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), k: f32| {
            (lerp(a.0, b.0, k), lerp(a.1, b.1, k), lerp(a.2, b.2, k), lerp(a.3, b.3, k))
        };
        if t < WINDUP {
            lerp4(rest, w, t / WINDUP)
        } else if t < WINDUP + STRIKE {
            lerp4(w, s, (t - WINDUP) / STRIKE)
        } else {
            lerp4(s, rest, (t - WINDUP - STRIKE) / RECOVER)
        }
    }
