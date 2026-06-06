#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Behavior {
    Aggressive,
    Pack,
    Defensive,
    Berserker,
    Venomous,
    Trickster,
    Healer,
    Charger,
}

impl Behavior {
    pub fn name(self) -> &'static str {
        match self {
            Behavior::Aggressive => "Aggressive",
            Behavior::Pack => "Pack",
            Behavior::Defensive => "Defensive",
            Behavior::Berserker => "Berserker",
            Behavior::Venomous => "Venomous",
            Behavior::Trickster => "Trickster",
            Behavior::Healer => "Healer",
            Behavior::Charger => "Charger",
        }
    }
}

pub struct Species {
    pub name: &'static str,
    pub emoji: &'static str,
    pub hp: f32,
    pub atk: f32,
    pub def: f32,
    pub spd: f32,
    pub behavior: Behavior,
    pub xp: f32,
    pub gold: f32,
}

pub struct Zone {
    pub name: &'static str,
    pub emoji: &'static str,
    pub min: u32,
    pub max: u32,
    pub species: &'static [usize],
}

use Behavior::*;

macro_rules! sp {
    ($n:expr, $e:expr, $hp:expr, $atk:expr, $def:expr, $spd:expr, $b:expr, $xp:expr, $g:expr) => {
        Species { name: $n, emoji: $e, hp: $hp, atk: $atk, def: $def, spd: $spd, behavior: $b, xp: $xp, gold: $g }
    };
}

pub const BOSS: usize = 15;
