//! Status effects: indices, per-fighter state and the pure rules (stacking, diminishing returns).
use crate::config::CombatTuning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Knock,
    Stagger,
    Stun,
    Confused,
    Slowed,
    Bleed,
    Poison,
}

impl Status {
    pub const ALL: [Status; 7] =
        [Status::Knock, Status::Stagger, Status::Stun, Status::Confused, Status::Slowed, Status::Bleed, Status::Poison];

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Status::Knock => "knock",
            Status::Stagger => "stagger",
            Status::Stun => "stun",
            Status::Confused => "confused",
            Status::Slowed => "slowed",
            Status::Bleed => "bleed",
            Status::Poison => "poison",
        }
    }
}

/// Per fighter status bookkeeping. `remaining` is in ticks, indexed by `Status::idx`.
#[derive(Clone, Debug, Default)]
pub struct StatusState {
    pub remaining: [u32; 7],
    pub stacks: [u8; 7],
    /// Source role index (0 hero, 1 monster) credited for damage over time.
    pub source: [usize; 7],
    pub stagger_imm: u32,
    pub stun_imm: u32,
    pub knock_imm: u32,
    /// Ticks at which stuns were received (for diminishing returns).
    pub stun_hist: Vec<u32>,
    pub bleed_timer: u32,
    pub poison_timer: u32,
    /// Knockback displacement per tick.
    pub knock_step: (f32, f32),
}

impl StatusState {
    pub fn tick_immunities(&mut self) {
        self.stagger_imm = self.stagger_imm.saturating_sub(1);
        self.stun_imm = self.stun_imm.saturating_sub(1);
        self.knock_imm = self.knock_imm.saturating_sub(1);
    }

    pub fn active(&self, s: Status) -> bool {
        self.remaining[s.idx()] > 0
    }
