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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DefenceTally {
    pub parry: u32,
    pub block: u32,
    pub dodge: u32,
    pub hit: u32,
    pub miss: u32,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub outcome: Outcome,
    pub ticks: u32,
    pub defence: [DefenceTally; 2],
    pub idle_guard_share: [f32; 2],
    /// Raw sum of per-threat credits (each at most 1.0, at most 6 threats) earned by each defender.
    pub defence_credit: [f32; 2],
}

const THREAT_SPEED: f32 = 2.5;
const THREAT_RANGE: f32 = 0.25;
const THREAT_HORIZON: f32 = 0.4;
const SWING_COOLDOWN: u32 = 12;
const IDLE_GUARD_WINDOW: u32 = 36;
const TURN_TICKS: u32 = 15;
const BLEED_START: u32 = 450;
const BLEED_PER_SEC: f32 = 0.5;
const SPAWN_A: f32 = 4.5;
const SPAWN_B: f32 = 9.5;
