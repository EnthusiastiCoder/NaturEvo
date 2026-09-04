//! Sensors (WS3 stub: constant zero observation).
use crate::fighter::Role;
use crate::matchup::Match;

pub const N_IN: usize = 40;

pub fn observe(_m: &Match, _role: Role) -> [f32; N_IN] {
    [0.0; N_IN]
}
