//! Deterministic tile map: zone biomes (Voronoi), grass encounter tiles, walls and A* pathing.
use crate::rng::Rng;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Floor,
    Grass,
    Wall,
    Camp,
}

pub struct Map {
    n: usize,
    tiles: Vec<Tile>,
    zones: Vec<u8>,
    anchors: Vec<(usize, usize)>,
}

const MAP_SALT: u64 = 0x4D41_505F_5345_4544;
const GRASS_SHARE: f64 = 0.35;
const WALL_SHARE: f64 = 0.08;

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
