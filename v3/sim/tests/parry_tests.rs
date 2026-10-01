// W2 sword-vs-sword resolution: deflect, clash, perfect parry. Geometry: attacker blade horizontal sweeping
// downward (3.4 m -> 2.8 m) across a vertical defender blade at x=3.0.
mod common;
use common::{arena, clashes, duel_pair, hits, seg};
use mech_sim::combat::{resolve_tick, ClashKind};
use mech_sim::robot::Robot;
