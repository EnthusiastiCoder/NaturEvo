//! Lethality model: no single hit ends a duel, a destroyed head blinds instead of killing,
//! death is torso destroyed or the part-health pool below KO_POOL_FRAC, duels last.
mod common;
use common::*;
use mech_sim::config::IMPACT_CAP;
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::robot::{Part, Role, State, KO_POOL_FRAC};
