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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    Grass,
    Elite,
    Boss,
}

impl Rank {
    pub const ALL: [Rank; 3] = [Rank::Grass, Rank::Elite, Rank::Boss];
    pub fn as_str(self) -> &'static str {
        match self {
            Rank::Grass => "grass",
            Rank::Elite => "elite",
            Rank::Boss => "boss",
        }
    }
}

pub const ELITE_DAMAGE_MULT: f32 = 1.15;
pub const BOSS_DAMAGE_MULT: f32 = 1.2;
