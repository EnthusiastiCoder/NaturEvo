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
