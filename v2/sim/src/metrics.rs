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

fn is_guard(s: CombatState) -> bool {
    matches!(s, CombatState::GuardRaise | CombatState::Blocking | CombatState::GuardRecover)
}

pub fn analyze(log: &FightLog, role: Role, outcome: Outcome) -> FightMetrics {
    let ri = role.index();
    let mut cells = [[[0.0f32; 6]; 4]; 2];
    let mut statuses_applied = [0u32; 7];
    let (mut punish_hits, mut parries) = (0u32, 0u32);
    let mut swings = [0u32; 3];
    let mut prev_state = CombatState::Idle;
    let n = log.frames.len();
    for f in &log.frames {
        let st = f.state[ri];
        let a = &f.action[ri];
        let d = ((f.pos[0].0 - f.pos[1].0).powi(2) + (f.pos[0].1 - f.pos[1].1).powi(2)).sqrt();
        let act = if matches!(st, CombatState::Staggered | CombatState::Stunned | CombatState::Knocked) {
            INVOLUNTARY
        } else if a.attack && !is_guard(st) {
            ATTACK
        } else if a.block || is_guard(st) {
            BLOCK
        } else if a.dodge {
            DODGE
        } else if a.fwd.abs() > MOVE_EPS || a.strafe.abs() > MOVE_EPS {
            MOVE
        } else {
            IDLE
        };
        cells[f.seen[ri] as usize][dist_bucket(d)][act] += 1.0;
        if st == CombatState::Windup && prev_state != CombatState::Windup && f.atk[ri] >= 0 {
            swings[(f.atk[ri] as usize).min(2)] += 1;
        }
        prev_state = st;
        for e in &f.ev {
            match e {
                Event::Hit { by, punish, fx, .. } if *by == ri => {
                    if *punish {
                        punish_hits += 1;
                    }
                    for k in fx {
                        statuses_applied[(*k).min(6)] += 1;
                    }
                }
                Event::Parry { by } if *by == ri => parries += 1,
                _ => {}
            }
        }
    }
    let inv = 1.0 / n.max(1) as f32;
    for c in cells.iter_mut().flatten().flatten() {
        *c *= inv;
    }
    let total: u32 = swings.iter().sum();
    let mut attack_kind_share = [0.0f32; 3];
    if total > 0 {
        for k in 0..3 {
            attack_kind_share[k] = swings[k] as f32 / total as f32;
        }
    }
    let draw = outcome == Outcome::Draw;
    FightMetrics { cells, statuses_applied, punish_hits, parries, attack_kind_share, draw, decisive: !draw }
}
