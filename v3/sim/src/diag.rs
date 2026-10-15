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

/// Summary (all normalised so fighters with different joint speeds compare).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Smooth {
    /// Mean |joint velocity| / joint max speed.
    pub vel_frac: f32,
    /// Share of joint-ticks moving at more than 90 percent of max speed.
    pub sat_frac: f32,
    /// Mean |change of joint velocity per tick| / (max speed): normalised jerk.
    pub jerk: f32,
    /// Mean |change of the raw target| per decision (normalised -1..1 units).
    pub tgt_delta: f32,
    /// Share of decisions on which a target moved in the opposite direction to its previous move.
    pub flip_rate: f32,
}
