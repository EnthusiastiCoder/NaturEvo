//! Deterministic move executor: (move, params) -> smooth keyframe joint targets.
//! State machine Idle -> Windup -> Active -> Recovery -> Idle. Active is never cancelled; Windup is cancelled only
//! by a Feint (before 60% of the windup) or by a threatened Backstep; the last 25% of Recovery accepts a chain.
//! The executor outputs the same `Action` as scripted.rs. Energy is only tracked here (`energy_spent`); the duel
//! applies it to the robot.
use crate::config::*;
use crate::moves::move_possible_id;
use crate::moveset::*;
use crate::robot::Robot;
use crate::scripted::Action;
