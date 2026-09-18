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
