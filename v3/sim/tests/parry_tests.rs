// W2 sword-vs-sword resolution: deflect, clash, perfect parry. Geometry: attacker blade horizontal sweeping
// downward (3.4 m -> 2.8 m) across a vertical defender blade at x=3.0.
mod common;
use common::{arena, clashes, duel_pair, hits, seg};
use mech_sim::combat::{resolve_tick, ClashKind};
use mech_sim::robot::Robot;

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}

/// vy: defender blade vertical velocity in m/s (positive = up, toward the descending attacker blade).
fn setup(vy: f32, moving_since: f32) -> (Robot, Robot) {
    let (mut a, mut d) = duel_pair();
    a.set_sword(seg(1.8, 3.4, 4.0, 3.4, 0.05), seg(1.8, 2.8, 4.0, 2.8, 0.05));
    a.sword_ang_vel = 10.0;
    a.strike_quality = 1.0;
    a.posture = 0.0;
    let dy = vy / 60.0;
    d.set_sword(seg(3.0, 1.5 - dy, 3.0, 3.7 - dy, 0.05), seg(3.0, 1.5, 3.0, 3.7, 0.05));
    d.sword_ang_vel = 8.0;
    d.sword_moving_since = moving_since;
    d.posture = 0.0;
    (a, d)
}

#[test]
fn defender_blade_moving_toward_attacker_deflects() {
    let (mut a, mut d) = setup(3.0, 0.5);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert_eq!(clashes(&ev), vec![ClashKind::Deflect], "{:?}", ev);
    assert!(hits(&ev).is_empty());
    approx(a.sword_ang_vel, -4.0, 1e-3); // reversed x0.4
    approx(a.posture, 25.0, 1e-3); // 25 x quality
    approx(d.posture, 5.0, 1e-3);
}

#[test]
fn static_defender_sword_is_a_plain_clash() {
    let (mut a, mut d) = setup(0.0, 0.0);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert_eq!(clashes(&ev), vec![ClashKind::Clash], "{:?}", ev);
    approx(a.sword_ang_vel, 5.0, 1e-3); // lose 50% speed
    approx(d.sword_ang_vel, 4.0, 1e-3);
    approx(a.posture, 15.0, 1e-3);
    approx(d.posture, 15.0, 1e-3);
}

#[test]
fn defender_blade_moving_away_is_a_plain_clash() {
    let (mut a, mut d) = setup(-3.0, 0.5);
    assert_eq!(clashes(&resolve_tick(&mut a, &mut d, &arena())), vec![ClashKind::Clash]);
}
