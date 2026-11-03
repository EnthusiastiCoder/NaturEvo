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
