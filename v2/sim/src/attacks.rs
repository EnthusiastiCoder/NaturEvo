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

/// One status applied by a hit. `chance` is always 1.0 (deterministic). For Knock `magnitude` is the
/// distance in tiles (duration comes from the combat tuning). `vulnerable_only` effects apply only
/// when the victim is Blocking or in Recovery.
#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    pub status: Status,
    pub chance: f32,
    pub duration: u32,
    pub magnitude: f32,
    pub vulnerable_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    pub name: &'static str,
    pub windup: u32,
    pub active: u32,
    pub recovery: u32,
    pub damage: f32,
    pub range: f32,
    pub half_arc: f32,
    pub stamina_cost: f32,
    /// Tiles moved forward during the active frames.
    pub lunge: f32,
    pub effects: Vec<Effect>,
}

fn fx(status: Status, duration: u32) -> Effect {
    Effect { status, chance: 1.0, duration, magnitude: 1.0, vulnerable_only: false }
}
fn knock(dist: f32) -> Effect {
    Effect { status: Status::Knock, chance: 1.0, duration: 0, magnitude: dist, vulnerable_only: false }
}
fn deg(d: f32) -> f32 {
    d * PI / 180.0
}
#[allow(clippy::too_many_arguments)]
fn mv(name: &'static str, w: u32, a: u32, r: u32, damage: f32, range: f32, half_arc_deg: f32, cost: f32, lunge: f32, effects: Vec<Effect>) -> Move {
    Move { name, windup: w, active: a, recovery: r, damage, range, half_arc: deg(half_arc_deg), stamina_cost: cost, lunge, effects }
}
