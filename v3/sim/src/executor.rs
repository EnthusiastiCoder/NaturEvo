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

#[derive(Clone, Debug)]
pub struct Executor {
    id: MoveId,
    params: MoveParams,
    phase: Phase,
    elapsed: f32,
    dur: [f32; 3],
    contact: bool,
    whiffed: bool,
    committed: bool,
    cancelled: bool,
    cancel_from: [f32; 12],
    start_pose: [f32; 12],
    spent: f32,
    hist1: [f32; 12],
    hist2: [f32; 12],
    last_pose: [f32; 12],
    threat: bool,
    riposte_open: bool,
    facing: f32,
}
