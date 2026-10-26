//! Measurement of what trained nets do in capped fights (diagnostics only, never read by fitness).
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::moveset::MoveId;
use crate::skillbot::is_attack;
use std::path::Path;

#[derive(Default, Clone, Debug)]
pub struct CapStats {
    pub fights: u32,
    pub capped: u32,
    /// Sums over capped fights (per fighter 0 and 1).
    pub gap_sum: f64,
    pub gap_n: f64,
    pub swings: [f64; 2],
    pub parries: [f64; 2],
    pub guards: [f64; 2],
    pub idles: [f64; 2],
    pub ticks: f64,
    /// Fights where fighter 0 / 1 made the first attack swing.
    pub first: [u32; 2],
    pub no_attack: u32,
}
