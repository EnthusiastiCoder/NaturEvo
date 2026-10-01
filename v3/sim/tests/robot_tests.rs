// W1 robot body. Assumed API additions beyond the plan contract are listed in v3/tests-index.md.
mod common;
use common::standing;
use mech_sim::config::IMPACT_CAP;
use mech_sim::robot::{Joint, Part, Robot, Role, State, DT, GUARD_TARGETS, STAND_TARGETS};
