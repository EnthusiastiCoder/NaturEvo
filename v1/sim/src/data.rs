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
    /// (species index, pick weight)
    pub species: &'static [(usize, u32)],
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
    sp!("Jungle King", "👑", 480.0, 38.0, 18.0, 14.0, Berserker, 500.0, 60.0),
];

pub static ZONES: [Zone; 9] = [
    Zone { name: "Base Camp", emoji: "⛺", min: 0, max: 0, species: &[(0, 1)] },
    Zone { name: "Jungle Edge", emoji: "🌿", min: 1, max: 4, species: &[(0, 4), (1, 3), (2, 1)] },
    Zone { name: "Dense Jungle", emoji: "🌳", min: 4, max: 9, species: &[(3, 4), (4, 3), (5, 2), (2, 1)] },
    Zone { name: "River Bank", emoji: "🏞️", min: 8, max: 13, species: &[(6, 4), (7, 3), (1, 2), (5, 1)] },
    Zone { name: "Misty Swamp", emoji: "🌫️", min: 12, max: 18, species: &[(8, 4), (7, 3), (4, 2), (6, 1)] },
    Zone { name: "Ancient Ruins", emoji: "🏛️", min: 16, max: 23, species: &[(9, 4), (10, 3), (11, 1)] },
    Zone { name: "Bat Caves", emoji: "🕳️", min: 21, max: 28, species: &[(12, 4), (13, 3), (11, 2), (9, 1)] },
    Zone { name: "Serpent Temple", emoji: "🛕", min: 26, max: 32, species: &[(14, 4), (13, 3), (10, 2), (11, 1)] },
    Zone { name: "King's Grove", emoji: "👑", min: 32, max: 34, species: &[(BOSS, 1)] },
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MoveEffect {
    None,
    Poison,
    Drain,
    Guard,
    Weaken,
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub name: &'static str,
    pub power: f32,
    pub weight: u32,
    pub effect: MoveEffect,
}

const fn mv(name: &'static str, power: f32, weight: u32, effect: MoveEffect) -> Move {
    Move { name, power, weight, effect }
}

use MoveEffect as E;

/// Weighted monster move lists, indexed like SPECIES.
pub static MOVES: [&[Move]; 16] = [
    &[mv("Gnaw", 1.0, 4, E::None), mv("Swarm Bite", 1.3, 2, E::None)],
    &[mv("Bite", 1.0, 3, E::None), mv("Venom Fang", 0.8, 3, E::Poison), mv("Constrict", 1.2, 1, E::Weaken)],
    &[mv("Scratch", 1.0, 4, E::None), mv("Snatch", 0.7, 2, E::Drain)],
    &[mv("Gore", 1.1, 4, E::None), mv("Trample", 1.6, 1, E::None)],
    &[mv("Bite", 1.0, 3, E::None), mv("Web Spit", 0.6, 2, E::Weaken), mv("Toxic Sting", 0.9, 2, E::Poison)],
    &[mv("Claw", 1.0, 4, E::None), mv("Pounce", 1.5, 2, E::None)],
    &[mv("Snap", 1.2, 3, E::None), mv("Death Roll", 1.7, 1, E::None), mv("Hide Plate", 0.3, 1, E::Guard)],
    &[mv("Tongue Lash", 0.9, 3, E::None), mv("Toxin Spray", 0.8, 2, E::Poison)],
    &[mv("Club Smash", 1.3, 3, E::None), mv("Regrowth", 0.5, 2, E::Drain)],
    &[mv("Rock Fist", 1.2, 3, E::None), mv("Stone Skin", 0.3, 2, E::Guard)],
    &[mv("Chill Touch", 1.0, 3, E::Drain), mv("Wail", 0.7, 2, E::Weaken)],
    &[mv("Slam", 1.2, 3, E::None), mv("Chest Pound", 0.6, 2, E::Weaken), mv("Rage Smash", 1.8, 1, E::None)],
    &[mv("Screech Bite", 0.9, 3, E::None), mv("Blood Sip", 0.7, 2, E::Drain)],
    &[mv("Talon Rake", 1.1, 3, E::None), mv("Lightning Dive", 1.6, 2, E::None)],
    &[mv("Fang Strike", 1.1, 3, E::Poison), mv("Coil Crush", 1.4, 2, E::None), mv("Hiss", 0.5, 1, E::Weaken)],
    &[mv("Royal Smash", 1.2, 4, E::None), mv("Roar", 0.6, 2, E::Weaken), mv("Crown Fury", 1.8, 1, E::None)],
];

pub const WEAPONS: [(&str, u32, i32); 7] = [
    ("Wooden Stick", 0, 0),
    ("Stone Axe", 60, 4),
    ("Iron Machete", 200, 9),
    ("Steel Blade", 500, 16),
    ("Obsidian Edge", 1100, 26),
    ("Thunder Spear", 2400, 40),
    ("Jungle Fang", 5000, 60),
];

pub const ARMORS: [(&str, u32, i32); 7] = [
    ("Rags", 0, 0),
    ("Leaf Wrap", 50, 3),
    ("Hide Vest", 180, 7),
    ("Bone Mail", 450, 12),
    ("Scale Armor", 1000, 19),
    ("Stoneplate", 2200, 28),
    ("Ancient Aegis", 4500, 40),
];

pub const POTION_COST: u32 = 25;

pub const MAX_STAGE: usize = 4;

/// Stage multipliers applied to monster hp, atk and def.
pub static STAGE_MULT: [f32; 5] = [1.0, 1.3, 1.7, 2.2, 3.0];

/// Five-stage evolution lines (A->B->C->D->E), indexed like SPECIES.
pub static EVOLUTIONS: [[&str; 5]; 16] = [
    ["Jungle Rat", "Dire Rat", "Plague Rat", "Rat Lord", "Rat King"],
    ["Vine Snake", "Thorn Snake", "Razor Viper", "Jade Serpent", "Emerald Wyrm"],
    ["Monkey Thief", "Monkey Bandit", "Monkey Outlaw", "Monkey Warlord", "Monkey Emperor"],
    ["Wild Boar", "Tusked Boar", "Ironhide Boar", "Warboar", "Boar Titan"],
    ["Giant Spider", "Hunter Spider", "Venom Weaver", "Silk Queen", "Spider Matriarch"],
    ["Jaguar", "Black Jaguar", "Shadow Panther", "Night Stalker", "Spirit Panther"],
    ["Crocodile", "Bog Crocodile", "Armored Croc", "Death Roller", "Leviathan Croc"],
    ["Poison Frog", "Toxic Frog", "Venom Toad", "Blight Toad", "Plague Toad King"],
    ["Swamp Troll", "Bog Troll", "Mire Brute", "Marsh Giant", "Swamp Colossus"],
    ["Stone Golem", "Granite Golem", "Iron Golem", "Obsidian Golem", "Titan Golem"],
    ["Ruin Wraith", "Grave Wraith", "Banshee", "Soul Reaver", "Wraith Sovereign"],
    ["Gorilla", "Silverback", "Rage Ape", "Ape Warlord", "Primal Kong"],
    ["Cave Bat", "Dire Bat", "Blood Bat", "Night Fiend", "Bat Overlord"],
    ["Thunder Raptor", "Storm Raptor", "Lightning Rex", "Tempest Rex", "Thunder Tyrant"],
    ["Ancient Serpent", "Elder Serpent", "Primeval Drake", "Dread Wyrm", "World Serpent"],
    ["Jungle King", "Jungle King", "Jungle King", "Jungle King", "Jungle King"],
];

pub fn stage_name(species: usize, stage: usize) -> &'static str {
    EVOLUTIONS[species][stage.min(MAX_STAGE)]
}
