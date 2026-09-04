//! A single real-time 1v1 duel: physics, combat state machine, logging.
use crate::config::ArenaConfig;
use crate::fighter::{Action, CombatState, Fighter, Role};
use crate::genome::Genome;
use crate::log::{FightLog, Frame};
use crate::net::Net;
use crate::sense::observe;
use crate::world::Arena;
use std::f32::consts::PI;

pub const WINDUP_TICKS: u32 = 6;
pub const ACTIVE_TICKS: u32 = 4;
pub const RECOVERY_TICKS: u32 = 8;
pub const ATTACK_RANGE: f32 = 1.2;
pub const ATTACK_HALF_ARC: f32 = PI / 4.0; // 90 degree arc
pub const DAMAGE: f32 = 12.0;
pub const BLOCK_HALF_ARC: f32 = PI / 3.0; // 120 degree arc
pub const BLOCK_REDUCTION: f32 = 0.8;
pub const BLOCK_STAMINA_COST: f32 = 15.0;
pub const DODGE_STAMINA_COST: f32 = 30.0;
pub const DODGE_TICKS: u32 = 8;
pub const DODGE_COOLDOWN: u32 = 45;
pub const STAMINA_REGEN_PER_SEC: f32 = 12.0;
