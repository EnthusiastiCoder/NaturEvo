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

/// allowed blocking kinds per zone (plan palette)
fn allowed(zone: usize) -> Vec<Tile> {
    match zone {
        0 => vec![Tile::Fence],
        1 | 2 => vec![Tile::Tree, Tile::Rock],
        3 => vec![Tile::Water, Tile::Rock],
        4 => vec![Tile::Water, Tile::Tree],
        5 | 6 => vec![Tile::Wall, Tile::Rock],
        7 => vec![Tile::Wall, Tile::Fence],
        8 => vec![Tile::Fence, Tile::Tree],
        _ => panic!("zone {}", zone),
    }
}
fn all(m: &Map) -> Vec<P> {
    let n = m.size();
    (0..n).flat_map(|y| (0..n).map(move |x| (x, y))).collect()
}
fn nbrs(n: usize, (x, y): P) -> Vec<P> {
    let mut v = vec![];
    if x > 0 {
        v.push((x - 1, y));
    }
    if y > 0 {
        v.push((x, y - 1));
    }
    if x + 1 < n {
        v.push((x + 1, y));
    }
    if y + 1 < n {
        v.push((x, y + 1));
    }
    v
}
fn flood(m: &Map) -> HashSet<P> {
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert(m.camp());
    q.push_back(m.camp());
    while let Some(p) = q.pop_front() {
        for nb in nbrs(m.size(), p) {
            if !m.tile(nb.0, nb.1).is_blocking() && seen.insert(nb) {
                q.push_back(nb);
            }
        }
    }
    seen
}
fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = Config::default();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
