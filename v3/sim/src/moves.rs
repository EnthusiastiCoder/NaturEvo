//! Move table, strike quality formula and move requirements.
//! Controllers output joint targets directly; moves are recognised post hoc.
use crate::config::IMPACT_CAP;
use crate::robot::{Joint, Part, Robot, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Slash,
    Thrust,
    ShieldBash,
    KickR,
    KickL,
    Guard,
    Parry,
    Dodge,
}

/// Required joints, proximal to distal. A disabled/destroyed part of any of these makes the move impossible.
pub fn required_joints(m: Move) -> &'static [Joint] {
    use Joint::*;
    match m {
        Move::Slash | Move::Thrust => &[HipR, Waist, ShoulderR, ElbowR],
        Move::ShieldBash => &[Waist, ShoulderL, ElbowL],
        Move::Guard => &[ShoulderL, ElbowL],
        Move::Parry => &[ShoulderR, ElbowR, Waist],
        Move::KickR => &[HipR, KneeR, AnkleR],
        Move::KickL => &[HipL, KneeL, AnkleL],
        Move::Dodge => &[HipL, HipR, KneeL, KneeR, AnkleL, AnkleR],
    }
}

/// Mean torque scale over the move's chain joints.
pub fn health_factor(r: &Robot, m: Move) -> f32 {
    let js = required_joints(m);
    js.iter().map(|j| r.torque_scale(*j)).sum::<f32>() / js.len() as f32
}

/// Timing: fraction of consecutive proximal->distal pairs whose peak times are in order.
/// Returns Tm = 0.6 + 0.4 * frac.
pub fn chain_quality(peak_times: &[f32]) -> f32 {
    if peak_times.len() < 2 {
        return 1.0;
    }
    let pairs = peak_times.len() - 1;
    let ok = peak_times.windows(2).filter(|w| w[0] <= w[1]).count();
    0.6 + 0.4 * ok as f32 / pairs as f32
}

/// Clamped cosine between strike velocity and contact normal.
pub fn alignment(v: [f32; 2], n: [f32; 2]) -> f32 {
    let lv = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let ln = (n[0] * n[0] + n[1] * n[1]).sqrt();
    if lv < 1e-9 || ln < 1e-9 {
        return 0.0;
    }
    ((v[0] * n[0] + v[1] * n[1]) / (lv * ln)).max(0.0)
}
