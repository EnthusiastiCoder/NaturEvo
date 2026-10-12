// Job 1: one source of truth for body geometry (shield plate perpendicular to the forearm, head, neck, log block).
mod common;
use common::*;
use mech_sim::combat::{resolve_tick, CombatEvent};
use mech_sim::config::*;
use mech_sim::duel::{Brain, Match};
use mech_sim::log::fight_json;
use mech_sim::robot::{Part, Role, DT, GUARD_TARGETS, STAND_TARGETS};
