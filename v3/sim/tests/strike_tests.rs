// A blade only wounds when the fighter's own swing drives it: walking onto a held blade is not a strike.
mod common;
use common::{arena, blade_sweep, duel_pair, hits, torso_center};
use mech_sim::combat::{own_blade_speed, resolve_tick};
use mech_sim::config::{MIN_STRIKE_SPEED, PHYS_DT};
use mech_sim::robot::{Part, STAND_TARGETS};

fn torso_edge(d: &mech_sim::robot::Robot) -> (f32, f32) {
    let tc = torso_center(d);
    (tc[0] - d.capsule(Part::Torso).r, tc[1])
}

#[test]
fn a_real_swing_still_wounds() {
    let (mut a, mut d) = duel_pair();
    let (edge, ty) = torso_edge(&d);
    blade_sweep(&mut a, edge - 0.3, edge + 0.1, ty - 0.9, ty + 0.9);
    assert!(own_blade_speed(&a) >= MIN_STRIKE_SPEED);
    assert_eq!(hits(&resolve_tick(&mut a, &mut d, &arena())).len(), 1);
}

#[test]
fn walking_a_held_blade_into_the_body_does_not_wound() {
    let (mut a, mut d) = duel_pair();
    // Bring the held blade (from the standing pose) through the defender's torso by walking only.
    let targets = STAND_TARGETS;
    for _ in 0..120 {
        a.apply_targets(&targets, PHYS_DT);
    }
    a.fk();
    let s = a.sword_capsule();
    let mid = (s.a[0] + s.b[0]) * 0.5;
    d.x = mid + 0.35;
    d.fk();
    let parked = common::seg(9.0, 0.3, 11.0, 0.3, 0.05);
    d.set_sword(parked, parked);
    d.set_shield(common::seg(9.0, 1.0, 9.0, 2.2, 0.05));
    let mut events = 0;
    for _ in 0..30 {
        a.apply_targets(&targets, PHYS_DT);
        a.apply_drive(1.0, PHYS_DT, 14.0); // full walking speed
        assert!(own_blade_speed(&a) < MIN_STRIKE_SPEED, "walking is not a swing: {}", own_blade_speed(&a));
        let ev = resolve_tick(&mut a, &mut d, &arena());
        events += hits(&ev).len();
    }
    assert_eq!(events, 0);
    assert!(d.hp_frac(Part::Torso) >= 1.0);
}

#[test]
fn walking_blades_together_is_not_a_clash() {
    let (mut a, mut d) = duel_pair();
    // d holds its blade still across a's path; a walks into it with the blade held (no swing).
    let still = common::seg(1.0, 0.5, 1.0, 2.5, 0.05);
    d.set_sword(still, still);
    let ahead = common::seg(0.7, 0.5, 0.7, 2.5, 0.05);
    let ahead2 = common::seg(0.75, 0.5, 0.75, 2.5, 0.05);
    a.set_sword(ahead, ahead2); // 3 m/s of displacement, all of it root motion
    a.x += 0.05;
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert!(common::clashes(&ev).is_empty(), "{:?}", ev);
}

#[test]
fn a_clash_pushes_the_fighters_apart_the_unsteadier_one_further() {
    let (mut a, mut d) = duel_pair();
    let still = common::seg(1.0, 0.5, 1.0, 2.5, 0.05);
    d.set_sword(still, still);
    d.parrying = true;
    a.set_sword(common::seg(0.6, 0.5, 0.6, 2.5, 0.05), common::seg(1.1, 0.5, 1.1, 2.5, 0.05));
    a.x = 3.0;
    a.posture = 60.0;
    d.posture = 0.0;
    let (ax, dx) = (a.x, d.x);
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert!(!common::clashes(&ev).is_empty(), "{:?}", ev);
    let (ma, md) = (ax - a.x, d.x - dx);
    assert!(ma > 0.0 && md > 0.0, "both move apart: {} {}", ma, md);
    assert!(ma > md, "the unsteadier attacker is pushed further: {} vs {}", ma, md);
}

/// Attacker slashes down across a held vertical defender blade at x=3.0; the defender torso is placed behind it.
fn beat_setup() -> (mech_sim::robot::Robot, mech_sim::robot::Robot) {
    let (mut a, mut d) = duel_pair();
    d.x = 3.4;
    d.fk();
    let tc = torso_center(&d);
    let blade = common::seg(3.0, tc[1] - 0.8, 3.0, tc[1] + 0.8, 0.05);
    d.set_sword(blade, blade);
    d.set_shield(common::seg(9.0, 1.0, 9.0, 2.2, 0.05));
    a.strike_quality = 1.0;
    // The attack sweeps from the left through the blade and on into the torso in one tick.
    a.set_sword(common::seg(1.5, tc[1] - 0.8, 1.5, tc[1] + 0.8, 0.05), common::seg(3.5, tc[1] - 0.8, 3.5, tc[1] + 0.8, 0.05));
    (a, d)
}

#[test]
fn a_swing_beats_aside_a_blade_that_is_merely_held() {
    let (mut a, mut d) = beat_setup();
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert_eq!(common::clashes(&ev).len(), 1, "{:?}", ev);
    assert_eq!(hits(&ev).len(), 1, "the swing carries on to the body: {:?}", ev);
}

#[test]
fn an_active_parry_stops_the_swing() {
    let (mut a, mut d) = beat_setup();
    d.parrying = true;
    let ev = resolve_tick(&mut a, &mut d, &arena());
    assert_eq!(common::clashes(&ev).len(), 1, "{:?}", ev);
    assert!(hits(&ev).is_empty(), "{:?}", ev);
}
