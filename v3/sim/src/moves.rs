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

/// Balance factor: kicks scale with margin; others 1.0, times 0.8 when posture > 70.
pub fn balance_factor(margin: f32, posture: f32, kick: bool) -> f32 {
    let mut b = if kick { (0.5 + margin / 0.5).clamp(0.5, 1.0) } else { 1.0 };
    if posture > 70.0 {
        b *= 0.8;
    }
    b
}

/// I = 0.5 m_eff v_n^2 (0.5 + 0.5 A) Tm H B, capped at IMPACT_CAP.
pub fn raw_impact(m_eff: f32, v_n: f32, a: f32, tm: f32, h: f32, b: f32) -> f32 {
    (0.5 * m_eff * v_n * v_n * (0.5 + 0.5 * a) * tm * h * b).min(IMPACT_CAP)
}

fn parts_ok(r: &Robot, parts: &[Part]) -> bool {
    parts.iter().all(|p| !r.is_disabled(*p))
}
