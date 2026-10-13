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
