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
