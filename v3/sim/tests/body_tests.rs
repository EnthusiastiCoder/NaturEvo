//! Bodies are solid: two fighters can neither overlap nor swap sides, and a staggered fighter recovers control quickly.
use mech_sim::arena::Arena;
use mech_sim::config::{BODY_MIN_SEP, STAGGER_LOCK, STAGGER_TIME};
use mech_sim::duel::{Brain, Match};
use mech_sim::robot::{Role, Robot, State, DT, STAND_TARGETS};
use mech_sim::stylebot::StyleId;
