// W2 moves: chain timing, impact formula, health/balance factors, move requirements.
mod common;
use mech_sim::config::IMPACT_CAP;
use mech_sim::moves::{alignment, balance_factor, chain_quality, health_factor, move_possible, raw_impact, required_joints, Move};
use mech_sim::robot::{Joint, Part, Robot, Role};

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}

#[test]
fn timing_in_order_is_1_reversed_is_0_6() {
    approx(chain_quality(&[0.10, 0.20, 0.30, 0.40]), 1.0, 1e-5);
    approx(chain_quality(&[0.40, 0.30, 0.20, 0.10]), 0.6, 1e-5);
    approx(chain_quality(&[0.1, 0.2, 0.4, 0.3]), 0.6 + 0.4 * 2.0 / 3.0, 1e-5);
}

#[test]
fn in_order_chain_gives_at_least_1_5x_impact_of_reversed() {
    let good = raw_impact(25.0, 1.5, 1.0, chain_quality(&[0.1, 0.2, 0.3, 0.4]), 1.0, 1.0);
    let bad = raw_impact(25.0, 1.5, 1.0, chain_quality(&[0.4, 0.3, 0.2, 0.1]), 1.0, 1.0);
    assert!(good < IMPACT_CAP, "pick inputs below the cap");
    assert!(good / bad >= 1.5, "{} / {}", good, bad);
}

#[test]
fn raw_impact_formula_exact() {
    // 0.5 * m_eff * v^2 * (0.5 + 0.5 A) * Tm * H * B
    let i = raw_impact(25.0, 2.0, 0.8, 0.9, 0.875, 1.0);
    approx(i, 0.5 * 25.0 * 4.0 * 0.9 * 0.9 * 0.875, 1e-3);
}

#[test]
fn raw_impact_is_capped() {
    approx(raw_impact(1000.0, 50.0, 1.0, 1.0, 1.0, 1.0), IMPACT_CAP, 1e-3);
}

#[test]
fn damaged_elbow_lowers_health_factor_and_impact_by_chain_mean() {
    // Section 2 formula: H = mean torque_scale over the chain joints; ElbowR at 25% hp (scale 0.5) in a 4-joint
    // chain gives 0.875 (the W2 prose says "halves": see plan issues in tests-index.md).
    let mut r = Robot::new(Role::Hero);
    approx(health_factor(&r, Move::Slash), 1.0, 1e-5);
    let m = r.max_hp(Part::FArmR);
    r.set_hp(Part::FArmR, 0.25 * m);
    let h = health_factor(&r, Move::Slash);
    approx(h, 0.875, 1e-4);
    let full = raw_impact(25.0, 1.5, 1.0, 1.0, 1.0, 1.0);
    let worn = raw_impact(25.0, 1.5, 1.0, 1.0, h, 1.0);
    approx(worn / full, 0.875, 1e-4);
}
