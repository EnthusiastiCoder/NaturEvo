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

/// Vertical attacker blade sweeping from `x_from` to `x_to` over one tick, covering [y_lo, y_hi].
pub fn blade_sweep(a: &mut Robot, x_from: f32, x_to: f32, y_lo: f32, y_hi: f32) {
    a.set_sword(seg(x_from, y_lo, x_from, y_hi, 0.05), seg(x_to, y_lo, x_to, y_hi, 0.05));
}

pub fn torso_center(d: &Robot) -> [f32; 2] {
    let c = d.capsule(Part::Torso);
    [(c.a[0] + c.b[0]) * 0.5, (c.a[1] + c.b[1]) * 0.5]
}

pub fn hits(ev: &[CombatEvent]) -> Vec<&HitEvent> {
    ev.iter().filter_map(|e| if let CombatEvent::Hit(h) = e { Some(h) } else { None }).collect()
}
pub fn clashes(ev: &[CombatEvent]) -> Vec<ClashKind> {
    ev.iter().filter_map(|e| if let CombatEvent::Clash { kind, .. } = e { Some(*kind) } else { None }).collect()
}
pub fn arena() -> Arena { Arena::flat(14.0) }
