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

/// Mean squared third derivative of the tip path (finite differences).
pub fn tip_jerk(tip: &[[f32; 2]], dt: f32) -> f32 {
    if tip.len() < 4 {
        return 0.0;
    }
    let k = 1.0 / (dt * dt * dt) as f64;
    let mut sum = 0.0f64;
    let n = tip.len() - 3;
    // Third differences below f32 quantisation of the positions are noise, not jerk.
    let scale = tip.iter().flat_map(|p| p.iter()).fold(1.0f32, |m, v| m.max(v.abs())) as f64;
    let floor = 16.0 * f32::EPSILON as f64 * scale;
    for i in 0..n {
        for a in 0..2 {
            let d = tip[i + 3][a] as f64 - 3.0 * tip[i + 2][a] as f64 + 3.0 * tip[i + 1][a] as f64 - tip[i][a] as f64;
            let j = if d.abs() < floor { 0.0 } else { d * k };
            sum += j * j;
        }
    }
    (sum / n as f64) as f32
}

/// Per-fighter raw records, filled by `Match` while it runs.
#[derive(Clone, Debug, Default)]
pub struct GraceRec {
    pub whiffs: u32,
    pub punish_hits: u32,
    pub open_windows: u32,
    pub parry_success: u32,
    pub feints: u32,
    pub baits: u32,
    pub recoveries_hit: u32,
    pub tip: Vec<[f32; 2]>,
    pub gap_n: u32,
    pub gap_sum: f64,
    pub gap_sq: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GraceStats {
    pub jerk: f32,
    pub spacing_std: f32,
    pub hits_per_swing: f32,
    pub whiff_rate: f32,
    pub punish_rate: f32,
    pub recovery_efficiency: f32,
    pub move_entropy: f32,
    pub usage: [f32; N_MOVES],
    pub parry_success_rate: f32,
    pub feint_bait_rate: f32,
    pub damage_per_energy: f32,
}
