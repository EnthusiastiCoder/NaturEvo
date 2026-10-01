// W3 defence: live-threat detection and exactly-once resolution.
mod common;
use common::{standing, torso_center};
use mech_sim::duel::{is_live_threat, Brain, Match, Resolution};
use mech_sim::robot::{Role, GUARD_TARGETS};
