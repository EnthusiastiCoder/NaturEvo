// W2 moves: chain timing, impact formula, health/balance factors, move requirements.
mod common;
use mech_sim::config::IMPACT_CAP;
use mech_sim::moves::{alignment, balance_factor, chain_quality, health_factor, move_possible, raw_impact, required_joints, Move};
use mech_sim::robot::{Joint, Part, Robot, Role};

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}
