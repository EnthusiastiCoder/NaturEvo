//! Jungle map: zone biomes (Voronoi), grass encounter tiles, obstacles and A* pathing.
//! Ported from v1/sim/src/map.rs (v1 untouched).
use crate::rng::Rng as BaseRng;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Floor,
    Grass,
    Wall,
    Camp,
    Rock,
    Fence,
    Tree,
    Water,
}

impl Tile {
    pub fn is_blocking(self) -> bool {
        matches!(self, Tile::Wall | Tile::Rock | Tile::Fence | Tile::Tree | Tile::Water)
    }
}

/// Integer helpers over the v2 splitmix RNG.
struct Rng(BaseRng);
impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(BaseRng::new(seed))
    }
    fn f(&mut self) -> f32 {
        self.0.f32()
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.0.next_u64() % ((hi - lo + 1) as u64)) as i32
    }
}

#[derive(Clone, Debug)]
pub struct Map {
    n: usize,
    tiles: Vec<Tile>,
    zones: Vec<u8>,
    anchors: Vec<(usize, usize)>,
}

const MAP_SALT: u64 = 0x4D41_505F_5345_4544;
const GRASS_SHARE: f64 = 0.35;
const OBSTACLE_SHARE: f64 = 0.095;
const CAMP_RING: usize = 2;
const GROVE_RING: usize = 4;
