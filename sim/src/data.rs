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

pub static SPECIES: [Species; 16] = [
    sp!("Jungle Rat", "🐀", 20.0, 6.0, 2.0, 10.0, Pack, 8.0, 1.0),
    sp!("Vine Snake", "🐍", 26.0, 7.0, 3.0, 12.0, Venomous, 10.0, 1.2),
    sp!("Monkey Thief", "🐒", 28.0, 8.0, 3.0, 16.0, Trickster, 11.0, 2.0),
    sp!("Wild Boar", "🐗", 40.0, 10.0, 5.0, 8.0, Charger, 13.0, 1.5),
    sp!("Giant Spider", "🕷️", 34.0, 11.0, 4.0, 13.0, Venomous, 14.0, 1.8),
    sp!("Jaguar", "🐆", 48.0, 14.0, 6.0, 15.0, Aggressive, 17.0, 2.0),
    sp!("Crocodile", "🐊", 70.0, 15.0, 10.0, 6.0, Defensive, 20.0, 2.4),
    sp!("Poison Frog", "🐸", 30.0, 9.0, 3.0, 14.0, Pack, 12.0, 1.4),
    sp!("Swamp Troll", "🧌", 110.0, 18.0, 9.0, 5.0, Healer, 26.0, 3.0),
    sp!("Stone Golem", "🗿", 130.0, 20.0, 16.0, 3.0, Defensive, 30.0, 3.5),
    sp!("Ruin Wraith", "👻", 80.0, 22.0, 6.0, 17.0, Trickster, 28.0, 3.2),
    sp!("Gorilla", "🦍", 120.0, 24.0, 10.0, 10.0, Berserker, 32.0, 3.6),
    sp!("Cave Bat", "🦇", 40.0, 12.0, 4.0, 20.0, Pack, 16.0, 1.6),
    sp!("Thunder Raptor", "🦖", 100.0, 28.0, 8.0, 22.0, Charger, 36.0, 4.0),
    sp!("Ancient Serpent", "🐉", 160.0, 30.0, 12.0, 14.0, Venomous, 40.0, 4.5),
    sp!("Jungle King", "👑", 420.0, 38.0, 18.0, 14.0, Berserker, 500.0, 60.0),
];
