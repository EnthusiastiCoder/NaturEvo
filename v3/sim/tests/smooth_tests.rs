// Job 2: smooth motion. Motors have a bounded acceleration and approach targets without overshoot, the net's
// action is low-passed, fitness carries a small jerk cost, and the smoothness metric itself is sane.
mod common;
use common::*;
use mech_sim::config::*;
use mech_sim::diag::SmoothAcc;
use mech_sim::duel::{smooth_toward, Brain, Match};
use mech_sim::fitness::{score_stats, MatchStats, W_JERK};
use mech_sim::robot::{Joint, Role, DT, STAND_TARGETS};
