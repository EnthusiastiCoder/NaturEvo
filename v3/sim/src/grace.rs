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

pub fn punish_rate(punish_hits: u32, open_windows: u32) -> f32 {
    if open_windows == 0 {
        0.0
    } else {
        (punish_hits as f32 / open_windows as f32).clamp(0.0, 1.0)
    }
}

pub fn move_usage_entropy(counts: &[u32; N_MOVES]) -> f32 {
    let total: u32 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let t = total as f32;
    counts.iter().filter(|&&c| c > 0).map(|&c| {
        let p = c as f32 / t;
        -p * p.ln()
    }).sum::<f32>().max(0.0)
}
