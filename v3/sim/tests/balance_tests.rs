// W1 balance and falling.
mod common;
use common::standing;
use mech_sim::robot::{Part, Robot, Role, State, DT, STAND_TARGETS};

fn step(r: &mut Robot, n: usize) {
    for _ in 0..n {
        r.apply_targets(&STAND_TARGETS, DT);
    }
}

#[test]
fn standing_margin_is_positive_and_state_idle() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    step(&mut r, 120);
    assert!(r.balance_margin() > 0.0, "margin {}", r.balance_margin());
    assert_eq!(r.state(), State::Idle);
}

#[test]
fn impulse_pushes_com_beyond_feet_negative_margin_then_fall() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.apply_impulse(12.0);
    let mut min_margin = f32::MAX;
    let mut fell = false;
    for _ in 0..120 {
        r.apply_targets(&STAND_TARGETS, DT);
        min_margin = min_margin.min(r.balance_margin());
        fell |= r.state() == State::Fallen;
    }
    assert!(min_margin < 0.0, "min margin {}", min_margin);
    assert!(fell, "margin < 0 for > 0.4 s must fall");
}

#[test]
fn small_impulse_is_recovered() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.apply_impulse(0.3);
    step(&mut r, 120);
    assert_eq!(r.state(), State::Idle);
}

#[test]
fn destroyed_leg_part_falls_immediately() {
    for p in [Part::ThighL, Part::ShinR, Part::FootL] {
        let mut r = standing(Role::Hero, 3.0, 1.0);
        r.set_hp(p, -0.5 * r.max_hp(p));
        step(&mut r, 2);
        assert_eq!(r.state(), State::Fallen, "{:?}", p);
    }
}

#[test]
fn disabled_leg_part_applies_half_metre_margin_penalty() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    let before = r.balance_margin();
    r.set_hp(Part::ThighL, 0.0);
    let after = r.balance_margin();
    assert!(((before - after) - 0.5).abs() < 0.1, "before {} after {}", before, after);
}

#[test]
fn fallen_robot_cannot_attack_takes_extra_damage_and_gets_up_after_1_5_s() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.knock_down();
    assert_eq!(r.state(), State::Fallen);
    assert!(!r.can_act());
    let base = Robot::new(Role::Hero).damage_part(Part::Torso, 10.0);
    let hit = r.clone().damage_part(Part::Torso, 10.0);
    assert!(((hit / base) - 1.5).abs() < 1e-3);
    step(&mut r, 85); // 1.42 s
    assert_eq!(r.state(), State::Fallen);
    step(&mut r, 10); // 1.58 s
    assert_eq!(r.state(), State::Idle);
    assert!(r.can_act());
}

#[test]
fn cannot_get_up_with_disabled_hip_or_both_knees() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.knock_down();
    r.set_hp(Part::ThighR, 0.0);
    step(&mut r, 240);
    assert_eq!(r.state(), State::Fallen);

    let mut k = standing(Role::Hero, 3.0, 1.0);
    k.knock_down();
    k.set_hp(Part::ShinL, 0.0);
    k.set_hp(Part::ShinR, 0.0);
    step(&mut k, 240);
    assert_eq!(k.state(), State::Fallen, "needs one working knee");
}
