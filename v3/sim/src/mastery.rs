//! Mastery measurement: per pairing over fixed seeds, how a fight looks (length, activity, binds, stagger loops, idling).
//! Diagnostics only; nothing here feeds back into fitness.
use crate::combat::swept_pair;
use crate::duel::{Brain, Match};
use crate::moveset::Phase;
use crate::robot::{Robot, State};
use crate::skillbot::is_attack;

#[derive(Clone, Debug, Default)]
pub struct Mastery {
    pub lens: Vec<u32>,
    pub capped: u32,
    pub swings: f64,
    pub hits: f64,
    pub distinct: f64,
    pub defences: f64,
    pub parries: f64,
    pub ticks: f64,
    pub both_stag: f64,
    pub bind: f64,
    pub hits_while_stag: f64,
    /// Ticks with both fighters in an Idle move at torso distance over 1.2 m (nobody doing anything).
    pub both_idle_far: f64,
    /// Longest run of consecutive both-idle ticks, summed over fights.
    pub longest_idle: f64,
    /// Blade-blade: ticks the blade capsules overlap at the end of the tick, ticks where the blade centre lines truly cross,
    /// of those, ticks with no clash event that tick, and the deepest penetration (m).
    pub blade_overlap: f64,
