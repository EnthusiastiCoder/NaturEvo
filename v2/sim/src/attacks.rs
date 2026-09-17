//! Attack definitions: hero moves and monster species movesets (data tables).
use crate::status::Status;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Hero,
    Minion,
    Rogue,
    Warrior,
    Mage,
}

impl Species {
    pub const ALL: [Species; 5] = [Species::Hero, Species::Minion, Species::Rogue, Species::Warrior, Species::Mage];
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Species::Hero => "hero",
            Species::Minion => "minion",
            Species::Rogue => "rogue",
            Species::Warrior => "warrior",
            Species::Mage => "mage",
        }
    }
}
