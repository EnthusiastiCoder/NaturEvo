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

#[test]
fn part_masses_hp_and_zones_match_table() {
    let r = Robot::new(Role::Hero);
    let rows = [
        (Part::Head, 40.0, 60.0, 0.30, 1.6),
        (Part::Torso, 300.0, 220.0, 0.45, 1.0),
        (Part::Pelvis, 180.0, 140.0, 0.40, 1.0),
        (Part::UArmL, 70.0, 80.0, 0.30, 0.8),
        (Part::FArmR, 50.0, 70.0, 0.30, 1.2),
        (Part::ThighL, 120.0, 110.0, 0.35, 0.9),
        (Part::ShinR, 90.0, 90.0, 0.30, 1.2),
        (Part::FootL, 40.0, 60.0, 0.30, 0.6),
    ];
    for (p, mass, hp, arm, zone) in rows {
        approx(r.part_mass(p), mass, 1e-3);
        approx(r.max_hp(p), hp, 1e-3);
        approx(r.armour(p), arm, 1e-4);
        approx(r.zone_mult(p), zone, 1e-4);
    }
    assert!(r.zone_mult(Part::Head) > r.zone_mult(Part::Torso));
    assert!(r.zone_mult(Part::Torso) > r.zone_mult(Part::FootL));
}

#[test]
fn fk_is_deterministic() {
    let t = [0.3, -0.2, 0.5, 0.1, 0.7, -0.4, 0.2, 0.2, 0.6, 0.1, -0.3, 0.3];
    let (mut a, mut b) = (Robot::new(Role::Hero), Robot::new(Role::Hero));
    for _ in 0..30 {
        a.apply_targets(&t, DT);
        b.apply_targets(&t, DT);
    }
    a.fk();
    b.fk();
    for p in Part::ALL {
        assert_eq!(a.capsule(p), b.capsule(p));
    }
    assert_eq!(a.com(), b.com());
}
