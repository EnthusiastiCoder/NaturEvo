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

pub fn seg(x1: f32, y1: f32, x2: f32, y2: f32, r: f32) -> Capsule {
    Capsule { a: [x1, y1], b: [x2, y2], r }
}

/// Attacker (facing +1, far left) and defender (facing -1). Both swords/shield parked far away and static.
pub fn duel_pair() -> (Robot, Robot) {
    let mut a = standing(Role::Hero, 0.0, 1.0);
    let mut d = standing(Role::Monster, 6.0, -1.0);
    let parked = seg(-5.0, 0.3, -3.0, 0.3, 0.05);
    a.set_sword(parked, parked);
    let parked2 = seg(9.0, 0.3, 11.0, 0.3, 0.05);
    d.set_sword(parked2, parked2);
    d.set_shield(seg(9.0, 1.0, 9.0, 2.2, 0.05));
    (a, d)
}
