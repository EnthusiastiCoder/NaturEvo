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

impl Map {
    pub fn generate(seed: u64, size: usize) -> Map {
        let n = size.max(8);
        let mut rng = Rng::new(seed ^ MAP_SALT);
        // anchors: camp in the corner, 8 zones on a jittered 3x3 grid, ordered by distance from camp
        let camp = (2usize, 2usize);
        let cell = n as f32 / 3.0;
        let jit = (n as f32 / 10.0) as i32;
        let mut cells: Vec<(usize, usize)> = Vec::new();
        for cy in 0..3usize {
            for cx in 0..3usize {
                if (cx, cy) == (0, 0) {
                    continue;
                }
                let jx = rng.range(-jit, jit);
                let jy = rng.range(-jit, jit);
                let x = ((cx as f32 + 0.5) * cell) as i32 + jx;
                let y = ((cy as f32 + 0.5) * cell) as i32 + jy;
                cells.push((x.clamp(3, n as i32 - 4) as usize, y.clamp(3, n as i32 - 4) as usize));
            }
        }
        let d2 = |a: (usize, usize), b: (usize, usize)| {
            let (dx, dy) = (a.0 as i64 - b.0 as i64, a.1 as i64 - b.1 as i64);
            dx * dx + dy * dy
        };
        cells.sort_by_key(|&a| (d2(a, camp), a.1, a.0));
        let mut anchors = vec![camp];
        anchors.extend(cells);

        // zones: nearest anchor (squared euclid, ties to the lower zone index)
        let mut zones = vec![0u8; n * n];
        for y in 0..n {
            for x in 0..n {
                let mut best = (i64::MAX, 0usize);
                for (z, &a) in anchors.iter().enumerate() {
                    let d = d2((x, y), a);
                    if d < best.0 {
                        best = (d, z);
                    }
                }
                zones[y * n + x] = best.1 as u8;
            }
        }

        // walls (never near an anchor) then grass (never in camp or grove)
        let near_anchor = |x: usize, y: usize| anchors.iter().any(|&a| a.0.abs_diff(x) <= 2 && a.1.abs_diff(y) <= 2);
        let wall_noise = noise_field(&mut rng, n, 4);
        let grass_noise = noise_field(&mut rng, n, 5);
        let elig_wall: Vec<bool> = (0..n * n).map(|i| !near_anchor(i % n, i / n)).collect();
        let walls = top_share(&wall_noise, &elig_wall, WALL_SHARE, None, 0);
        let elig_grass: Vec<bool> = (0..n * n)
            .map(|i| !walls[i] && zones[i] != 0 && zones[i] != 8 && !near_anchor(i % n, i / n))
            .collect();
        let grass = top_share(&grass_noise, &elig_grass, GRASS_SHARE, Some(&zones), 45);
        let mut tiles: Vec<Tile> = (0..n * n)
            .map(|i| if walls[i] { Tile::Wall } else if grass[i] { Tile::Grass } else { Tile::Floor })
            .collect();
        tiles[camp.1 * n + camp.0] = Tile::Camp;

        let mut m = Map { n, tiles, zones, anchors };
        // reachability: carve an L corridor to any cut-off anchor, then wall off unreachable floor
        for _ in 0..16 {
            let seen = m.flood();
            let Some(z) = (1..9).find(|&z| {
                let a = m.anchors[z];
                !seen[a.1 * n + a.0]
            }) else {
                break;
            };
            let (a, b) = (m.anchors[z], m.anchors[z - 1]);
            let (mut x, mut y) = a;
            while (x, y) != b {
                if x != b.0 {
                    x = if x < b.0 { x + 1 } else { x - 1 };
                } else {
                    y = if y < b.1 { y + 1 } else { y - 1 };
                }
                if m.tiles[y * n + x] == Tile::Wall {
                    m.tiles[y * n + x] = Tile::Floor;
                }
            }
        }
        let seen = m.flood();
        for i in 0..n * n {
            if !seen[i] {
                m.tiles[i] = Tile::Wall;
            }
        }
        m
    }

    fn flood(&self) -> Vec<bool> {
        let n = self.n;
        let mut seen = vec![false; n * n];
        let c = self.anchors[0];
        let mut q = VecDeque::new();
        seen[c.1 * n + c.0] = true;
        q.push_back(c);
        while let Some(p) = q.pop_front() {
            for nb in self.nbrs(p) {
                if self.tiles[nb.1 * n + nb.0] != Tile::Wall && !seen[nb.1 * n + nb.0] {
                    seen[nb.1 * n + nb.0] = true;
                    q.push_back(nb);
                }
            }
        }
        seen
    }

    fn nbrs(&self, (x, y): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        let n = self.n;
        let mut v = [(usize::MAX, usize::MAX); 4];
        if y > 0 {
            v[0] = (x, y - 1);
        }
        if x > 0 {
            v[1] = (x - 1, y);
        }
        if x + 1 < n {
            v[2] = (x + 1, y);
        }
        if y + 1 < n {
            v[3] = (x, y + 1);
        }
        v.into_iter().filter(|p| p.0 != usize::MAX)
    }
