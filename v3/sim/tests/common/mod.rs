// Shared helpers for v3 sim tests. APIs assumed: see v3/tests-index.md.
#![allow(dead_code)]
use mech_sim::arena::Arena;
use mech_sim::combat::{CombatEvent, ClashKind, HitEvent};
use mech_sim::robot::{Capsule, Part, Robot, Role, DT, STAND_TARGETS};

pub fn standing(role: Role, x: f32, facing: f32) -> Robot {
    let mut r = Robot::new(role);
    r.x = x;
    r.facing = facing;
    for _ in 0..10 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    r
}
