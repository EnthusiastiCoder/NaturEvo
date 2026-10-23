//! Grace metrics: how smooth, economical and well-timed a fighter's play is. MEASURED ONLY, never rewarded.
use crate::duel::Match;
use crate::moveset::{MoveId, N_MOVES};
use crate::robot::DT;
use crate::skillbot::is_attack;

pub fn whiff_rate(attack_starts: u32, whiffs: u32) -> f32 {
    if attack_starts == 0 {
        0.0
    } else {
        (whiffs as f32 / attack_starts as f32).clamp(0.0, 1.0)
    }
}

pub fn hits_per_swing(attack_starts: u32, body_hits: u32) -> f32 {
    if attack_starts == 0 {
        0.0
    } else {
        body_hits as f32 / attack_starts as f32
    }
}
