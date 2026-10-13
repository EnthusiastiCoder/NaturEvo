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

fn poke(a: &mut mech_sim::robot::Robot, q: [f32; 2], w: [f32; 2]) {
    // short blade at q moving by 0.4 m along unit w in one tick
    let s = |o: f32| seg(q[0] + o * w[0] - 0.05, q[1] + o * w[1], q[0] + o * w[0] + 0.05, q[1] + o * w[1], 0.05);
    a.set_sword(s(0.0), s(0.4));
}
fn blocks(ev: &[CombatEvent]) -> usize { ev.iter().filter(|e| matches!(e, CombatEvent::Block { .. })).count() }

#[test]
fn sword_on_plate_face_blocks_but_edge_and_arm_behind_do_not() {
    let (mut a, _) = duel_pair();
    let mut d = guarded();
    d.set_sword(seg(9.0, 0.3, 11.0, 0.3, 0.05), seg(9.0, 0.3, 11.0, 0.3, 0.05));
    let f = d.capsule(Part::FArmL);
    let p = d.shield_capsule();
    let fa = sub(f.b, f.a);
    let fl = (fa[0] * fa[0] + fa[1] * fa[1]).sqrt();
    let axis = [fa[0] / fl, fa[1] / fl];
    let pc = [(p.a[0] + p.b[0]) / 2.0, (p.a[1] + p.b[1]) / 2.0];
    let pdir = sub(p.b, p.a);
    let pl = (pdir[0] * pdir[0] + pdir[1] * pdir[1]).sqrt();
    let pu = [pdir[0] / pl, pdir[1] / pl];
    // face: blade crosses the plate centre moving along the forearm axis (plate normal)
    poke(&mut a, [pc[0] - axis[0] * 0.2, pc[1] - axis[1] * 0.2], axis);
    let ev = resolve_tick(&mut a, &mut d.clone(), &arena());
    assert_eq!(blocks(&ev), 1, "face hit must block: {:?}", ev);
    // edge: well beyond the plate end, same direction
    let q = [pc[0] + pu[0] * (SHIELD_LEN / 2.0 + 0.3) - axis[0] * 0.2, pc[1] + pu[1] * (SHIELD_LEN / 2.0 + 0.3) - axis[1] * 0.2];
    poke(&mut a, q, axis);
    let ev = resolve_tick(&mut a, &mut d.clone(), &arena());
    assert_eq!(blocks(&ev), 0, "edge hit must not block: {:?}", ev);
    // arm behind: on the forearm near the elbow (old parallel capsule would have blocked), sideways swing
    let q = [f.a[0] + axis[0] * 0.1 + pu[0] * 0.5, f.a[1] + axis[1] * 0.1 + pu[1] * 0.5];
    poke(&mut a, q, [-pu[0], -pu[1]]);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert_eq!(blocks(&ev), 0, "arm hit near the elbow must not block: {:?}", ev);
}

#[test]
fn head_matches_config_and_neck_hit_damages_head() {
    let d = standing(Role::Monster, 6.0, -1.0);
    assert!((d.capsule(Part::Head).r - HEAD_R).abs() < 1e-6);
    let n = d.neck_capsule();
    assert!((n.r - NECK_R).abs() < 1e-6);
    let mut a = standing(Role::Hero, 0.0, 1.0);
    let mut d = d;
    d.set_sword(seg(9.0, 0.3, 11.0, 0.3, 0.05), seg(9.0, 0.3, 11.0, 0.3, 0.05));
    d.set_shield(seg(9.0, 1.0, 9.0, 2.2, 0.05));
    let mid = [n.a[0] + (n.b[0] - n.a[0]) * 0.9, n.a[1] + (n.b[1] - n.a[1]) * 0.9]; // upper neck, outside the torso cap
    // vertical blade passing through the neck from the attacker side
    a.set_sword(seg(mid[0] - 0.8, mid[1] - 0.03, mid[0] - 0.8, mid[1] + 0.03, 0.02), seg(mid[0] + 0.0, mid[1] - 0.03, mid[0], mid[1] + 0.03, 0.02));
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert!(hits(&ev).iter().any(|h| h.part == Part::Head), "{:?}", ev);
}
