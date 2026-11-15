// W2 combat: swept collision, hit location and direction, knockback, hit-stop, block, caps.
mod common;
use common::{arena, blade_sweep, clashes, duel_pair, hits, seg, torso_center};
use mech_sim::combat::{hitstop_ticks, resolve_tick, swept_hit, CombatEvent};
use mech_sim::config::IMPACT_CAP;
use mech_sim::robot::Part;

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}

/// Edge x of the defender torso facing the attacker, plus its centre y.
fn torso_edge(d: &mech_sim::robot::Robot) -> (f32, f32) {
    let tc = torso_center(d);
    (tc[0] - d.capsule(Part::Torso).r, tc[1])
}

#[test]
fn swept_blade_does_not_tunnel_at_24_m_per_s() {
    // thin vertical target at x=0 (radius 0.1); blade (radius 0.05) jumps from x=-0.2 to x=+0.2 in one 1/60 s tick
    let target = seg(0.0, 0.0, 0.0, 2.0, 0.1);
    let prev = seg(-0.2, 0.5, -0.2, 1.5, 0.05);
    let cur = seg(0.2, 0.5, 0.2, 1.5, 0.05);
    let c = swept_hit(prev, cur, target).expect("mid-tick crossing must register");
    assert!(c.normal[0] > 0.9, "normal points into the target (+x), got {:?}", c.normal);
    assert!(c.point[0] > -0.2 && c.point[0] < 0.2);
    assert!((0.5..=1.5).contains(&c.point[1]));
}

#[test]
fn swept_blade_that_never_reaches_misses() {
    let target = seg(0.0, 0.0, 0.0, 2.0, 0.1);
    assert!(swept_hit(seg(-1.0, 3.0, -1.0, 4.0, 0.05), seg(1.0, 3.0, 1.0, 4.0, 0.05), target).is_none());
    assert!(swept_hit(seg(-1.0, 0.5, -1.0, 1.5, 0.05), seg(-0.5, 0.5, -0.5, 1.5, 0.05), target).is_none());
}

#[test]
fn resolve_tick_reports_hit_part_point_and_direction() {
    let (mut a, mut d) = duel_pair();
    let (edge, ty) = torso_edge(&d);
    blade_sweep(&mut a, edge - 0.5, edge + 0.1, ty - 0.9, ty + 0.9);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    let h = hits(&ev);
    assert_eq!(h.len(), 1, "{:?}", ev);
    let h = h[0];
    assert_eq!(h.part, Part::Torso);
    assert!(h.point[0] > edge - 0.3 && h.point[0] < edge + 0.3, "point {:?} edge {}", h.point, edge);
    assert!((ty - 0.9..=ty + 0.9).contains(&h.point[1]));
    assert!(h.normal[0] > 0.99 && h.normal[1].abs() < 0.1, "direction = relative velocity at p, got {:?}", h.normal);
    assert!(h.impact > 0.0 && h.damage > 0.0);
    approx(h.zone, 1.0, 1e-4);
    assert!(d.hp_frac(Part::Torso) < 1.0);
}

#[test]
fn hit_adds_posture_equal_to_impact_times_0_05() {
    let (mut a, mut d) = duel_pair();
    let (edge, ty) = torso_edge(&d);
    blade_sweep(&mut a, edge - 0.05, edge + 0.1, ty - 0.9, ty + 0.9);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    let h = hits(&ev)[0];
    approx(d.posture, h.impact * 0.05, 1e-3);
}

#[test]
fn knockback_follows_normal_x_and_scales_with_impact() {
    let kb = |w: f32| {
        let (mut a, mut d) = duel_pair();
        let (edge, ty) = torso_edge(&d);
        blade_sweep(&mut a, edge + 0.1 - w, edge + 0.1, ty - 0.9, ty + 0.9);
        let ev = resolve_tick(&mut a, &mut d, &arena());
        let h = hits(&ev)[0].clone();
        (h.impact, h.knockback)
    };
    let (i_slow, k_slow) = kb(0.05); // 3 m/s
    let (i_fast, k_fast) = kb(0.1); // 6 m/s
    assert!(k_slow > 0.0, "attacker on the left pushes defender +x");
    assert!(i_fast > i_slow);
    assert!(k_fast > k_slow);
}

#[test]
fn hitstop_is_3_to_6_ticks_and_monotonic() {
    let mut last = 0;
    for i in 0..=20 {
        let t = hitstop_ticks(IMPACT_CAP * i as f32 / 20.0);
        assert!((3..=6).contains(&t), "{} at step {}", t, i);
        assert!(t >= last);
        last = t;
    }
    assert!(hitstop_ticks(IMPACT_CAP) > hitstop_ticks(0.0));
}

#[test]
fn hit_event_carries_hitstop_in_range() {
    let (mut a, mut d) = duel_pair();
    let (edge, ty) = torso_edge(&d);
    blade_sweep(&mut a, edge - 0.3, edge + 0.1, ty - 0.9, ty + 0.9);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert!((3..=6).contains(&hits(&ev)[0].hitstop_ticks));
}

#[test]
fn impact_is_capped() {
    let (mut a, mut d) = duel_pair();
    let (edge, ty) = torso_edge(&d);
    blade_sweep(&mut a, edge - 20.0, edge + 0.1, ty - 0.9, ty + 0.9); // absurd speed
    let ev = resolve_tick(&mut a, &mut d, &arena());
    approx(hits(&ev)[0].impact, IMPACT_CAP, 1e-3);
}

fn shield_setup(d: &mut mech_sim::robot::Robot, edge: f32, ty: f32) -> f32 {
    let xp = edge - 0.15;
    d.set_shield(seg(xp, ty - 0.6, xp, ty + 0.6, 0.05));
    xp
}

#[test]
fn shield_between_sword_and_torso_blocks_with_85_percent_reduction() {
    // baseline: same swing without a shield in the way
    let (mut a0, mut d0) = duel_pair();
    let (edge, ty) = torso_edge(&d0);
    let xp = edge - 0.15;
    blade_sweep(&mut a0, xp - 0.25, xp + 0.25, ty - 0.9, ty + 0.9);
    let raw = hits(&resolve_tick(&mut a0, &mut d0, &arena()))[0].impact;

    let (mut a, mut d) = duel_pair();
    shield_setup(&mut d, edge, ty);
    blade_sweep(&mut a, xp - 0.25, xp + 0.25, ty - 0.9, ty + 0.9);
    let before = d.shield_integrity;
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert!(hits(&ev).is_empty(), "blocked swing must not hit the body: {:?}", ev);
    let b = ev.iter().find_map(|e| if let CombatEvent::Block { raw, reduced, .. } = e { Some((*raw, *reduced)) } else { None }).expect("block event");
    approx(b.0, raw, raw * 0.01);
    approx(b.1, 0.15 * b.0, 1e-3);
    approx(d.shield_integrity, (before - 0.3 * b.0).max(0.0), 1e-2);
    assert!(d.hp_frac(Part::FArmL) < 1.0, "shield arm takes the residual");
    assert!(d.hp_frac(Part::Torso) == 1.0);
    approx(a.posture, 10.0, 1e-3);
    approx(d.posture, 10.0 * (1.0 - 0.5), 1e-2); // full integrity: 10 * (1 - 1.0 * 0.5)
}

#[test]
fn dropped_shield_or_empty_energy_means_no_block_and_hit_lands() {
    for variant in 0..2 {
        let (mut a, mut d) = duel_pair();
        let (edge, ty) = torso_edge(&d);
        let xp = shield_setup(&mut d, edge, ty);
        if variant == 0 {
            d.shield_integrity = 0.0;
        } else {
            d.energy = 0.0;
        }
        blade_sweep(&mut a, xp - 0.25, xp + 0.25, ty - 0.9, ty + 0.9);
        let ev = resolve_tick(&mut a, &mut d, &arena());
        assert_eq!(hits(&ev).len(), 1, "variant {} -> {:?}", variant, ev);
        assert!(!ev.iter().any(|e| matches!(e, CombatEvent::Block { .. })));
        let _ = clashes(&ev);
    }
}

#[test]
fn head_hit_does_more_damage_than_foot_hit_for_same_swing() {
    let dmg = |part: Part| {
        let (mut a, mut d) = duel_pair();
        let c = d.capsule(part);
        let (cx, cy) = ((c.a[0] + c.b[0]) * 0.5, (c.a[1] + c.b[1]) * 0.5);
        let edge = cx - c.r;
        blade_sweep(&mut a, edge - 0.05, edge + 0.1, cy - 0.15, cy + 0.15);
        let ev = resolve_tick(&mut a, &mut d, &arena());
        let h = hits(&ev);
        assert!(!h.is_empty(), "{:?} not hit", part);
        h[0].damage / h[0].impact
    };
    assert!(dmg(Part::Head) > dmg(Part::FootL));
}
