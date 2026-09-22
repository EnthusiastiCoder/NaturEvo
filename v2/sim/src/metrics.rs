//! Behaviour metrics: a pure function of a fight log. Action shares are bucketed by whether the
//! fighter saw the enemy and by distance, so "blocks while the enemy is unseen" is directly measurable.
use crate::fighter::{CombatState, Role};
use crate::log::{Event, FightLog};
use crate::matchup::Outcome;

pub const ATTACK: usize = 0;
pub const BLOCK: usize = 1;
pub const DODGE: usize = 2;
pub const MOVE: usize = 3;
pub const IDLE: usize = 4;
pub const INVOLUNTARY: usize = 5;
