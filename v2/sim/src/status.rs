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
