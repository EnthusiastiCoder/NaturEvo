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

impl SmoothAcc {
    pub fn new() -> SmoothAcc {
        SmoothAcc::default()
    }

    /// Call once per physics tick after the robot has stepped.
    pub fn tick(&mut self, r: &Robot) {
        let mut v = [0.0f32; 12];
        for j in Joint::ALL {
            let i = j as usize;
            v[i] = r.joint_vel[i];
            let m = j.max_speed();
            self.vel_frac += (v[i].abs() / m) as f64;
            if v[i].abs() > 0.9 * m {
                self.sat += 1.0;
            }
            if let Some(p) = &self.prev_vel {
                self.jerk += ((v[i] - p[i]).abs() / m) as f64;
            }
        }
        self.prev_vel = Some(v);
        self.ticks += 1;
    }

    /// Call whenever the policy emits a new raw target vector.
    pub fn decision(&mut self, t: &[f32; 12]) {
        if let Some(p) = &self.prev_tgt {
            for i in 0..12 {
                let d = t[i] - p[i];
                self.tgt_delta += d.abs() as f64;
                if d.abs() > 0.02 {
                    if self.prev_dt[i].abs() > 0.02 {
                        self.flip_den += 1;
                        if d * self.prev_dt[i] < 0.0 {
                            self.flips += 1;
                        }
                    }
                    self.prev_dt[i] = d;
                }
            }
            self.decisions += 1;
        }
        self.prev_tgt = Some(*t);
    }

    pub fn finish(&self) -> Smooth {
        let jt = (self.ticks.max(1) * 12) as f64;
        Smooth {
            vel_frac: (self.vel_frac / jt) as f32,
            sat_frac: (self.sat / jt) as f32,
            jerk: (self.jerk / jt) as f32,
            tgt_delta: (self.tgt_delta / (self.decisions.max(1) * 12) as f64) as f32,
            flip_rate: self.flips as f32 / self.flip_den.max(1) as f32,
        }
    }
}

use crate::duel::{Brain, Match};
