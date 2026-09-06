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
