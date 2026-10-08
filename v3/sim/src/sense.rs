//! Sensing: the network input vector, in the fighter's own facing frame (forward = +x).
//! The opponent is always visible (no FOV). A damaged head adds deterministic noise to opponent
//! inputs; a disabled head blinds the fighter (opponent inputs are zero).
use crate::arena::Arena;
use crate::config::PHYS_DT;
use crate::rng::Rng;
use crate::robot::{Capsule, Joint, Part, Robot, State};
use std::ops::Range;

/// Number of normalised network inputs.
pub const N_IN: usize = 104;

/// Opponent key points (8 points, x then y each) in own frame, relative to own torso centre.
pub const OPP_POINTS: Range<usize> = 64..80;
/// Opponent sword tip velocity (2), speed toward me, time to contact, threat height.
pub const OPP_SWORD: Range<usize> = 80..85;
/// Opponent part hp fractions (13).
pub const OPP_HP: Range<usize> = 85..98;

const POS_SCALE: f32 = 4.0;
const VEL_SCALE: f32 = 10.0;

fn mid(c: &Capsule) -> [f32; 2] {
    [(c.a[0] + c.b[0]) * 0.5, (c.a[1] + c.b[1]) * 0.5]
}

fn norm_angle(r: &Robot, j: Joint) -> f32 {
    let (lo, hi) = j.limits();
    ((r.joint_angle(j) - lo) / (hi - lo) * 2.0 - 1.0).clamp(-1.0, 1.0)
}
