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

#[test]
fn joint_limits_clamp_and_targets_map_to_range() {
    for j in Joint::ALL {
        let (lo, hi) = j.limits();
        assert!(lo < hi);
        let mut r = Robot::new(Role::Hero);
        let mut hi_t = STAND_TARGETS;
        hi_t[j as usize] = 5.0; // out-of-range target is clamped
        for _ in 0..240 {
            r.apply_targets(&hi_t, DT);
        }
        assert!(r.joint_angle(j) <= hi + 1e-4, "{:?} above limit", j);
        let mut lo_t = STAND_TARGETS;
        lo_t[j as usize] = -5.0;
        for _ in 0..240 {
            r.apply_targets(&lo_t, DT);
        }
        assert!(r.joint_angle(j) >= lo - 1e-4, "{:?} below limit", j);
    }
    let (lo, hi) = Joint::ElbowR.limits();
    approx(lo, 0.0, 1e-4);
    approx(hi, 140f32.to_radians(), 1e-3);
    let (slo, shi) = Joint::ShoulderR.limits();
    approx(slo, (-30f32).to_radians(), 1e-3);
    approx(shi, 170f32.to_radians(), 1e-3);
}

#[test]
fn joint_to_part_mapping() {
    assert_eq!(Joint::ElbowR.part(), Part::FArmR);
    assert_eq!(Joint::ShoulderR.part(), Part::UArmR);
    assert_eq!(Joint::HipL.part(), Part::ThighL);
    assert_eq!(Joint::KneeR.part(), Part::ShinR);
    assert_eq!(Joint::AnkleL.part(), Part::FootL);
    assert_eq!(Joint::Waist.part(), Part::Torso);
    assert_eq!(Joint::Neck.part(), Part::Head);
}

#[test]
fn torque_scale_follows_hp() {
    let mut r = Robot::new(Role::Hero);
    approx(r.torque_scale(Joint::ElbowR), 1.0, 1e-5);
    set_frac(&mut r, Part::FArmR, 0.25);
    approx(r.torque_scale(Joint::ElbowR), 0.5, 1e-4);
    set_frac(&mut r, Part::FArmR, 0.1);
    approx(r.torque_scale(Joint::ElbowR), 0.25, 1e-5);
    set_frac(&mut r, Part::FArmR, 0.6);
    approx(r.torque_scale(Joint::ElbowR), 1.0, 1e-5);
    approx(r.hp_frac(Part::FArmR), 0.6, 1e-5);
}

#[test]
fn armour_eff_and_tankiness() {
    let mut r = Robot::new(Role::Hero);
    approx(r.armour_eff(Part::Torso), 0.45, 1e-5);
    set_frac(&mut r, Part::Torso, 0.25);
    approx(r.armour_eff(Part::Torso), 0.45 * 0.55, 1e-4);
}

#[test]
fn damage_formula_exact_and_more_on_damaged_part() {
    let mut fresh = Robot::new(Role::Hero);
    let d0 = fresh.damage_part(Part::Torso, 10.0); // impact * zone * (1 - armour_eff)
    approx(d0, 10.0 * 1.0 * (1.0 - 0.45), 1e-3);
    approx(fresh.hp(Part::Torso), 220.0 - d0, 1e-3);
    let mut worn = Robot::new(Role::Hero);
    set_frac(&mut worn, Part::Torso, 0.25);
    let d1 = worn.damage_part(Part::Torso, 10.0);
    approx(d1, 10.0 * (1.0 - 0.45 * 0.55), 1e-3);
    assert!(d1 > d0);
}

#[test]
fn zone_multiplier_scales_damage_and_impact_is_capped() {
    let head = Robot::new(Role::Hero).damage_part(Part::Head, 10.0);
    let torso = Robot::new(Role::Hero).damage_part(Part::Torso, 10.0);
    let foot = Robot::new(Role::Hero).damage_part(Part::FootL, 10.0);
    let uarm = Robot::new(Role::Hero).damage_part(Part::UArmL, 10.0);
    assert!(head > torso && torso > foot);
    assert!(head > uarm, "high zone beats limb for the same impact");
    let capped = Robot::new(Role::Hero).damage_part(Part::Torso, IMPACT_CAP);
    let over = Robot::new(Role::Hero).damage_part(Part::Torso, IMPACT_CAP * 10.0);
    approx(capped, over, 1e-3);
}

#[test]
fn disabled_at_zero_hp_joint_goes_limp() {
    let mut r = Robot::new(Role::Hero);
    set_frac(&mut r, Part::FArmR, 0.0);
    assert!(r.is_disabled(Part::FArmR));
    assert!(!r.is_destroyed(Part::FArmR));
    let mut healthy = Robot::new(Role::Hero);
    let mut t = STAND_TARGETS;
    t[Joint::ElbowR as usize] = 1.0;
    for _ in 0..90 {
        r.apply_targets(&t, DT);
        healthy.apply_targets(&t, DT);
    }
    assert!(
        (r.joint_angle(Joint::ElbowR) - healthy.joint_angle(Joint::ElbowR)).abs() > 0.3,
        "limp joint ignores target"
    );
}

#[test]
fn destroyed_at_minus_half_max_detaches_distal_children_and_mass() {
    let mut r = Robot::new(Role::Hero);
    let m0 = r.total_mass();
    r.set_hp(Part::FootR, -0.5 * r.max_hp(Part::FootR));
    assert!(r.is_destroyed(Part::FootR));
    approx(m0 - r.total_mass(), 40.0, 1e-3);

    let mut r = Robot::new(Role::Hero);
    let (m0, c0) = (r.total_mass(), r.com());
    r.set_hp(Part::ShinL, -0.5 * r.max_hp(Part::ShinL));
    assert!(r.is_destroyed(Part::ShinL));
    assert!(r.is_detached(Part::ShinL) && r.is_detached(Part::FootL), "distal child detaches too");
    approx(m0 - r.total_mass(), 130.0, 1e-3);
    assert_ne!(r.com(), c0);
}

#[test]
fn overkill_accumulates_into_destruction() {
    let mut r = Robot::new(Role::Hero);
    for _ in 0..200 {
        r.damage_part(Part::FootR, IMPACT_CAP);
        if r.is_destroyed(Part::FootR) {
            break;
        }
    }
    assert!(r.is_destroyed(Part::FootR));
    assert!(r.hp(Part::FootR) <= -0.5 * r.max_hp(Part::FootR) + 1e-3);
}

#[test]
fn weapon_and_shield_are_dropped_with_the_forearm() {
    let mut r = Robot::new(Role::Hero);
    assert!(r.has_sword() && r.has_shield());
    r.set_hp(Part::FArmR, -0.5 * r.max_hp(Part::FArmR));
    assert!(!r.has_sword() && r.has_shield());
    r.set_hp(Part::FArmL, -0.5 * r.max_hp(Part::FArmL));
    assert!(!r.has_shield());
}

#[test]
// Updated for the lethality model: a destroyed head blinds instead of killing (see lethality_tests.rs).
fn destroyed_torso_is_ko() {
    for p in [Part::Torso] {
        let mut r = standing(Role::Hero, 3.0, 1.0);
        assert_ne!(r.state(), State::Ko);
        r.set_hp(p, -0.5 * r.max_hp(p));
        r.apply_targets(&STAND_TARGETS, DT);
        assert_eq!(r.state(), State::Ko, "{:?}", p);
    }
}

#[test]
fn guard_drains_energy_about_5_per_second() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.energy = 100.0;
    for _ in 0..30 {
        r.apply_targets(&GUARD_TARGETS, DT);
    }
    assert!(r.guard_raised());
    let e = r.energy;
    for _ in 0..60 {
        r.apply_targets(&GUARD_TARGETS, DT);
    }
    let drop = e - r.energy;
    assert!((4.0..=6.0).contains(&drop), "guard drains ~5/s, got {}", drop);
}

#[test]
fn fast_motors_cost_energy_idle_regenerates() {
    let mut s = standing(Role::Hero, 3.0, 1.0);
    s.energy = 100.0;
    for i in 0..60 {
        let mut t = STAND_TARGETS;
        let v = if (i / 10) % 2 == 0 { 1.0 } else { -1.0 };
        t[Joint::ShoulderR as usize] = v;
        t[Joint::ElbowR as usize] = v;
        t[Joint::Waist as usize] = v;
        s.apply_targets(&t, DT);
    }
    assert!(s.energy < 95.0, "fast motors cost energy, got {}", s.energy);

    let mut i = standing(Role::Hero, 3.0, 1.0);
    i.energy = 50.0;
    for _ in 0..60 {
        i.apply_targets(&STAND_TARGETS, DT);
    }
    approx(i.energy, 58.0, 1.5);
}

#[test]
fn zero_energy_caps_torque_and_deactivates_shield() {
    let mut z = Robot::new(Role::Hero);
    z.energy = 0.0;
    for j in Joint::ALL {
        assert!(z.torque_scale(j) <= 0.3 + 1e-5, "{:?}", j);
    }
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.energy = 0.0;
    for _ in 0..10 {
        r.apply_targets(&GUARD_TARGETS, DT);
    }
    assert!(!r.shield_active());
}

#[test]
fn posture_decays_after_half_second() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.posture = 0.0;
    r.add_posture(70.0);
    for _ in 0..28 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    approx(r.posture, 70.0, 0.5);
    for _ in 0..62 {
        r.apply_targets(&STAND_TARGETS, DT); // ~1.0 s of decay at 15/s
    }
    approx(r.posture, 55.0, 2.0);
}

#[test]
fn posture_100_staggers_with_exposed_damage_window() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.posture = 0.0;
    r.add_posture(100.0);
    assert_eq!(r.state(), State::Staggered);
    approx(r.posture, 40.0, 1e-3);
    let base = Robot::new(Role::Hero).damage_part(Part::Torso, 10.0);
    let hit = r.clone().damage_part(Part::Torso, 10.0);
    approx(hit / base, 1.5, 1e-3);
    assert!(!r.can_act());
    for _ in 0..40 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    assert_eq!(r.state(), State::Staggered, "0.8 s = 48 ticks");
    for _ in 0..15 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    assert_eq!(r.state(), State::Idle);
    assert!(r.can_act());
}
