// Downward-thrust finisher (plunge): Overhead and Thrust reach a fallen opponent when the executor knows the
// target is down (Executor::set_target_down). Tests first: see plans/reports for the CTO decision.
use mech_sim::combat::{resolve_tick_with, CombatEvent, Mods};
use mech_sim::executor::Executor;
use mech_sim::moveset::*;
use mech_sim::robot::{Joint, Part, Role, Robot, DT};
use mech_sim::rules;
