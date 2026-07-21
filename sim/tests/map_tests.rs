//! Map generation, A* and data-table tests (plan 2026-10-21_hero-map-walking, workstream 1).
//! Written from the plan before any implementation.
//!
//! API assumed (names picked where the plan is silent):
//!   jungle_sim::map::{Map, Tile, map_body}
//!   Map::generate(seed: u64, size: usize) -> Map
//!   Map::size(&self) -> usize                      (square map, w == h)
//!   Map::tile(&self, x: usize, y: usize) -> Tile   (Tile::{Floor, Grass, Wall, Camp}, Copy + PartialEq + Debug)
//!   Map::zone_of(&self, x, y) -> usize             (0..=8)
//!   Map::passable(&self, x, y) -> bool             (false for walls and out of bounds)
//!   Map::camp(&self) -> (usize, usize)
//!   Map::anchors(&self) -> &[(usize, usize)]       (9 entries, zone index order)
//!   Map::path(&self, from: (usize, usize), to: (usize, usize)) -> Option<Vec<(usize, usize)>>
//!       excludes `from`, includes `to`; from == to gives Some(vec![])
//!   Map::nearest_tile_of_zone(&self, from: (usize, usize), zone: usize) -> Option<(usize, usize)>
//!   map::map_body(&Map) -> String                  (JSON of GET /api/map)
use jungle_sim::data::{Move, MOVES, SPECIES, ZONES};
use jungle_sim::map::{map_body, Map, Tile};
use std::collections::VecDeque;
