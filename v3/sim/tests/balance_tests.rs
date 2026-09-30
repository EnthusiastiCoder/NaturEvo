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
