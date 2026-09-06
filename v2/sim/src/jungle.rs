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

/// Obstacle kinds allowed per zone (first kind is the default for pocket fills).
const PALETTE: [&[Tile]; 9] = [
    &[Tile::Fence],
    &[Tile::Tree, Tile::Rock],
    &[Tile::Tree, Tile::Rock],
    &[Tile::Water, Tile::Rock],
    &[Tile::Water, Tile::Tree],
    &[Tile::Wall, Tile::Rock],
    &[Tile::Rock, Tile::Wall],
    &[Tile::Wall, Tile::Fence],
    &[Tile::Fence, Tile::Tree],
];
/// Kind-noise below this picks the first palette kind.
const FIRST_KIND: [f32; 9] = [1.0, 0.56, 0.56, 0.5, 0.5, 0.5, 0.56, 0.5, 0.4];

fn kind_of(zone: usize, v: f32) -> Tile {
    let pal = PALETTE[zone];
    if pal.len() == 1 || v < FIRST_KIND[zone] {
        pal[0]
    } else {
        pal[1]
    }
}

/// A spacing minimum defined for a 64-wide map, scaled to `n` (never below 6).
pub fn scaled_min(base: usize, n: usize) -> usize {
    (base * n / 64).max(6)
}

/// Smooth value noise: a coarse random lattice, smoothstep-interpolated.
fn noise_field(rng: &mut Rng, n: usize, cell: usize) -> Vec<f32> {
    let g = n / cell + 2;
    let lat: Vec<f32> = (0..g * g).map(|_| rng.f()).collect();
    let mut out = Vec::with_capacity(n * n);
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 / cell as f32, y as f32 / cell as f32);
            let (x0, y0) = (fx as usize, fy as usize);
            let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
            let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
            let a = lat[y0 * g + x0];
            let b = lat[y0 * g + x0 + 1];
            let c = lat[(y0 + 1) * g + x0];
            let d = lat[(y0 + 1) * g + x0 + 1];
            let top = a + (b - a) * sx;
            let bot = c + (d - c) * sx;
            out.push(top + (bot - top) * sy);
        }
    }
    out
}

/// Marks the `share` highest-noise tiles among `eligible` (ties broken by index).
/// With `groups`, the quota is applied per group, with at least `min_take` (capped at 70%) each.
fn top_share(noise: &[f32], eligible: &[bool], share: f64, groups: Option<&[u8]>, min_take: usize) -> Vec<bool> {
    let mut out = vec![false; noise.len()];
    for gid in 0..9u8 {
        let mut idx: Vec<usize> =
            (0..noise.len()).filter(|&i| eligible[i] && groups.map_or(gid == 0, |g| g[i] == gid)).collect();
        idx.sort_by(|&a, &b| noise[b].partial_cmp(&noise[a]).unwrap().then(a.cmp(&b)));
        let want = (idx.len() as f64 * share).round() as usize;
        let take = want.max(min_take.min(idx.len() * 7 / 10));
        for &i in idx.iter().take(take) {
            out[i] = true;
        }
    }
    out
}
