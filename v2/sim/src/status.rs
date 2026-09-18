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
