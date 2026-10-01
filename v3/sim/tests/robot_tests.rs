// W1 robot body. Assumed API additions beyond the plan contract are listed in v3/tests-index.md.
mod common;
use common::standing;
use mech_sim::config::IMPACT_CAP;
use mech_sim::robot::{Joint, Part, Robot, Role, State, DT, GUARD_TARGETS, STAND_TARGETS};

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}
fn set_frac(r: &mut Robot, p: Part, f: f32) {
    let m = r.max_hp(p);
    r.set_hp(p, m * f);
}

#[test]
fn has_13_parts_and_12_joints() {
    assert_eq!(Part::ALL.len(), 13);
    assert_eq!(Joint::ALL.len(), 12);
}
