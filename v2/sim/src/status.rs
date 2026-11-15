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

    /// Stun duration after diminishing returns: base * 0.5^n, n = stuns in the previous window.
    pub fn stun_duration(&self, base: u32, now: u32, t: &CombatTuning) -> u32 {
        let n = self.stun_hist.iter().filter(|&&at| now.saturating_sub(at) < t.stun_dr_window).count() as i32;
        let d = (base as f32 * 0.5f32.powi(n)).round() as u32;
        d.max(t.stun_dr_floor.min(base))
    }

    /// Speed multiplier from slowed stacks.
    pub fn speed_mult(&self, t: &CombatTuning) -> f32 {
        let n = self.stacks[Status::Slowed.idx()] as i32;
        if self.active(Status::Slowed) && n > 0 {
            t.slowed_factor.powi(n).max(t.slowed_floor)
        } else {
            1.0
        }
    }
}

/// Integer hash of (match seed, fighter index, window) used for confusion; never touches the shared Rng.
pub fn confusion_hash(seed: u64, index: usize, window: u32) -> u64 {
    let mut z = seed ^ ((index as u64) << 56) ^ ((window as u64) << 20) ^ 0x9E37_79B9_7F4A_7C15;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
