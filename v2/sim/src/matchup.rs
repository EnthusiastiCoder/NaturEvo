//! A single real-time 1v1 duel: physics, combat state machine, logging.
use crate::config::ArenaConfig;
use crate::fighter::{Action, CombatState, Fighter, Role};
use crate::genome::Genome;
use crate::log::{FightLog, Frame};
use crate::net::Net;
use crate::sense::observe;
use crate::world::Arena;
use std::f32::consts::PI;
