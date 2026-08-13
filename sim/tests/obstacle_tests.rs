//! Obstacle types (plan 2026-10-21_hero-map-walking, Addendum 3 part A). Written before any implementation.
//!
//! API picked where the plan is silent:
//!   Tile::{Floor, Grass, Camp, Wall, Rock, Fence, Tree, Water}  (Copy + PartialEq + Debug)
//!   Tile::is_blocking(self) -> bool       (true for Wall, Rock, Fence, Tree, Water)
//!   Map::with_tile(&self, x, y, Tile) -> Map   (copy of the map with one tile replaced; used for the hand-built block)
//!   Boss tile = Map::anchors()[8]. Map::passable(x, y) == !tile.is_blocking() (false out of bounds).
//!   Obstacle share = blocking tiles / (size * size - 1).
//!   /api/map legend names: Wall, Rock, Fence, Tree, Water, Floor, Grass, Camp.
mod common;
use common::parse;
use jungle_sim::config::Config;
use jungle_sim::map::{map_body, Map, Tile};
use jungle_sim::report::run_body;
use jungle_sim::server::route;
use jungle_sim::sim::simulate_with;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

type P = (usize, usize);

const BLOCKING: [Tile; 5] = [Tile::Wall, Tile::Rock, Tile::Fence, Tile::Tree, Tile::Water];
