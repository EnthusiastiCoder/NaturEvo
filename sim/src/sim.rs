use crate::data::*;
use crate::rng::Rng;

const EVENT_CAP: usize = 300_000;
const MAX_LEVEL: u32 = 40;
const MAX_ROUNDS: u32 = 60;
const BOSS_LEVEL: u32 = 32;

#[derive(Clone, Debug)]
pub struct HeroSnap {
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub xp: u32,
    pub gold: u32,
    pub potions: u32,
    pub zone: usize,
    pub weapon: usize,
    pub armor: usize,
}

#[derive(Clone, Debug)]
pub struct MonSnap {
    pub species: usize,
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
}
