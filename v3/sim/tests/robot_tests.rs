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

#[test]
fn standing_mass_and_height() {
    let r = standing(Role::Hero, 3.0, 1.0);
    let m = r.total_mass();
    assert!((1100.0..=1300.0).contains(&m), "total mass {}", m);
    let top = Part::ALL
        .iter()
        .map(|p| {
            let c = r.capsule(*p);
            c.a[1].max(c.b[1]) + c.r
        })
        .fold(0.0f32, f32::max);
    assert!((3.8..=4.2).contains(&top), "height {}", top);
    let bottom = Part::ALL
        .iter()
        .map(|p| {
            let c = r.capsule(*p);
            c.a[1].min(c.b[1]) - c.r
        })
        .fold(f32::MAX, f32::min);
    approx(bottom, 0.0, 0.1);
}
