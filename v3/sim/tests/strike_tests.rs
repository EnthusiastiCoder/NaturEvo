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
