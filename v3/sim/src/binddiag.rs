//! Measurement of blade binds and body overlap in fights (diagnostics only, never read by fitness).
use crate::combat::swept_pair;
use crate::duel::{Brain, Match};
use crate::robot::{Part, Robot};

#[derive(Clone, Debug, Default)]
pub struct BindStats {
    pub fights: u32,
    pub ticks: f64,
    /// Ticks with the two blades touching.
    pub blade_touch: f64,
    /// Ticks with the blades touching and both blade tips slower than 1 m/s (a static bind).
    pub static_bind: f64,
    /// Longest run of consecutive static-bind ticks, summed over fights.
    pub longest_bind: f64,
    /// Ticks with the two bodies (head, torso, pelvis capsules) overlapping.
    pub body_overlap: f64,
    /// Number of times the fighters swapped sides.
    pub swaps: f64,
    pub swings: f64,
    pub hits: f64,
    /// Hits dealt while the attacker was not in an attack move's windup or active phase (blade drift, chip).
    pub drift_hits: f64,
}
