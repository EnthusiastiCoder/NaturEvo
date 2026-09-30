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
