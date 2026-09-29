// W1 balance and falling.
mod common;
use common::standing;
use mech_sim::robot::{Part, Robot, Role, State, DT, STAND_TARGETS};

fn step(r: &mut Robot, n: usize) {
    for _ in 0..n {
        r.apply_targets(&STAND_TARGETS, DT);
    }
}
