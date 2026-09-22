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

/// Distance bucket upper bounds (tiles): close, mid, far; anything beyond is very far.
pub const DIST_CLOSE: f32 = 1.2;
pub const DIST_MID: f32 = 4.0;
pub const DIST_FAR: f32 = 12.0;
/// Intent below this magnitude counts as standing still.
pub const MOVE_EPS: f32 = 0.1;

#[derive(Clone, Debug, PartialEq)]
pub struct FightMetrics {
    /// Share of ticks, indexed [seen (0 unseen, 1 seen)][distance bucket][action].
    pub cells: [[[f32; 6]; 4]; 2],
    /// Distinct hits applying each status (index = `Status::idx`), credited to this role.
    pub statuses_applied: [u32; 7],
    pub punish_hits: u32,
    pub parries: u32,
    /// Share of swings started per move index.
    pub attack_kind_share: [f32; 3],
    pub draw: bool,
    pub decisive: bool,
}

impl FightMetrics {
    /// Share of all ticks spent blocking while the enemy was not seen.
    pub fn block_unseen_share(&self) -> f32 {
        self.cells[0].iter().map(|d| d[BLOCK]).sum()
    }
    /// Share of ticks the enemy was seen.
    pub fn seen_share(&self) -> f32 {
        self.cells[1].iter().flatten().sum()
    }
    /// Share of all ticks spent blocking.
    pub fn block_share(&self) -> f32 {
        self.cells.iter().flatten().map(|d| d[BLOCK]).sum()
    }
    /// Total statuses applied (count of distinct status applications).
    pub fn statuses_total(&self) -> u32 {
        self.statuses_applied.iter().sum()
    }
}

pub fn dist_bucket(d: f32) -> usize {
    if d <= DIST_CLOSE {
        0
    } else if d <= DIST_MID {
        1
    } else if d <= DIST_FAR {
        2
    } else {
        3
    }
}
