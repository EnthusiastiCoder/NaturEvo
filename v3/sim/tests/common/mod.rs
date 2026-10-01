// Shared helpers for v3 sim tests. APIs assumed: see v3/tests-index.md.
#![allow(dead_code)]
use mech_sim::arena::Arena;
use mech_sim::combat::{CombatEvent, ClashKind, HitEvent};
use mech_sim::robot::{Capsule, Part, Robot, Role, DT, STAND_TARGETS};
