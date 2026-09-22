//! Behaviour metrics: a pure function of a fight log. Action shares are bucketed by whether the
//! fighter saw the enemy and by distance, so "blocks while the enemy is unseen" is directly measurable.
use crate::fighter::{CombatState, Role};
use crate::log::{Event, FightLog};
use crate::matchup::Outcome;
