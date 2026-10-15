//! Motion smoothness diagnostics: how jerky is a fighter's joint motion and how often do its targets change.
use crate::robot::{Joint, Robot};

/// Accumulates per-tick joint statistics for one fighter.
#[derive(Clone, Debug, Default)]
pub struct SmoothAcc {
    ticks: u32,
    vel_frac: f64,
    sat: f64,
    jerk: f64,
    prev_vel: Option<[f32; 12]>,
    decisions: u32,
    tgt_delta: f64,
    flips: u32,
    flip_den: u32,
    prev_tgt: Option<[f32; 12]>,
    prev_dt: [f32; 12],
}
