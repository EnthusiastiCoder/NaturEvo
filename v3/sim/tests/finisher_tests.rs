// Downward-thrust finisher (plunge): Overhead and Thrust reach a fallen opponent when the executor knows the
// target is down (Executor::set_target_down). Tests first: see plans/reports for the CTO decision.
use mech_sim::combat::{resolve_tick_with, CombatEvent, Mods};
use mech_sim::executor::Executor;
use mech_sim::moveset::*;
use mech_sim::robot::{Joint, Part, Role, Robot, DT};
use mech_sim::rules;

mod common;
use common::*;

fn fast() -> MoveParams { MoveParams { reach: 1.0, speed: 1.0, arc: 0.5, delay: 0.0 } }

fn fallen_at(x: f32) -> Robot {
    let mut d = standing(Role::Monster, x, -1.0);
    d.knock_down();
    d.fk();
    d
}
