// Job 1: one source of truth for body geometry (shield plate perpendicular to the forearm, head, neck, log block).
mod common;
use common::*;
use mech_sim::combat::{resolve_tick, CombatEvent};
use mech_sim::config::*;
use mech_sim::duel::{Brain, Match};
use mech_sim::log::fight_json;
use mech_sim::robot::{Part, Role, DT, GUARD_TARGETS, STAND_TARGETS};

fn guarded() -> mech_sim::robot::Robot {
    let mut d = standing(Role::Monster, 6.0, -1.0);
    for _ in 0..90 {
        d.apply_targets(&GUARD_TARGETS, DT);
    }
    d
}
fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] { [a[0] - b[0], a[1] - b[1]] }

#[test]
fn shield_plate_is_perpendicular_to_forearm_and_sized_from_config() {
    for t in [&STAND_TARGETS, &GUARD_TARGETS] {
        let mut d = standing(Role::Monster, 6.0, -1.0);
        for _ in 0..90 { d.apply_targets(t, DT); }
        let f = d.capsule(Part::FArmL);
        let p = d.shield_capsule();
        let fa = sub(f.b, f.a);
        let pa = sub(p.b, p.a);
        let dotp = fa[0] * pa[0] + fa[1] * pa[1];
        assert!(dotp.abs() < 1e-4, "plate not perpendicular: {}", dotp);
        let l = (pa[0] * pa[0] + pa[1] * pa[1]).sqrt();
        assert!((l - SHIELD_LEN).abs() < 1e-4);
        let c = [(p.a[0] + p.b[0]) / 2.0, (p.a[1] + p.b[1]) / 2.0];
        let want = [f.a[0] + fa[0] * SHIELD_ALONG, f.a[1] + fa[1] * SHIELD_ALONG];
        assert!((c[0] - want[0]).abs() < 1e-3 && (c[1] - want[1]).abs() < 1e-3);
    }
}
