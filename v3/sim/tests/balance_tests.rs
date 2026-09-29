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
