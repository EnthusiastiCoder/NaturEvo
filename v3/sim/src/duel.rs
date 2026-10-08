//! Duel loop: two robots, brains, combat per tick, threat tracking, anti-stalemate bleed, outcome.
use crate::arena::Arena;
use crate::combat::{resolve_tick, CombatEvent, ClashKind};
use crate::config::{DECISION_TICKS, IMPACT_REF, MAX_TICKS, PHYS_DT};
use crate::genome::Genome;
use crate::log::{self, f2, FightLog};
use crate::moves::raw_impact;
use crate::net::Net;
use crate::robot::{Capsule, Joint, Part, Robot, Role, State, STAND_TARGETS};
use crate::scripted::{self, Action, Scripted};
use crate::sense::observe;

/// Who controls a fighter.
#[derive(Clone, Debug)]
pub enum Brain {
    Net(Genome),
    /// Stands still.
    Dummy,
    /// Approaches and swings at random intervals.
    Scripted,
    /// Holds the given normalised joint targets.
    Fixed([f32; 12]),
}

enum Ctl {
    Net(Box<Net>, Action),
    Dummy,
    Scripted(Box<Scripted>),
    Fixed(Action),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    WinA,
    WinB,
    Draw,
}

/// How a live threat ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Hit,
    Parried,
    Blocked,
    Dodged,
    Miss,
}

#[derive(Clone, Debug)]
pub struct Threat {
    pub id: u32,
    pub attacker: usize,
    pub defender: usize,
    pub start_tick: u32,
    pub resolution: Option<Resolution>,
    /// Impact the swing could have delivered (from its peak tip speed).
    pub potential: f32,
    start_x: f32,
    start_angles: [f32; 12],
    moved: f32,
    flexed: f32,
    quiet: u32,
}
