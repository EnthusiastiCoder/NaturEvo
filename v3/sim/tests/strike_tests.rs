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
