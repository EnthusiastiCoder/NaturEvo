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
