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

        // obstacles (never near an anchor), kind by biome palette
        let near_anchor = |x: usize, y: usize| anchors.iter().any(|&a| a.0.abs_diff(x) <= 2 && a.1.abs_diff(y) <= 2);
        let obst_noise = noise_field(&mut rng, n, 4);
        let grass_noise = noise_field(&mut rng, n, 5);
        let kind_noise = noise_field(&mut rng, n, 6);
        let gate_side = rng.range(0, 3);
        let elig_obst: Vec<bool> = (0..n * n).map(|i| zones[i] != 0 && !near_anchor(i % n, i / n)).collect();
        let blocked = top_share(&obst_noise, &elig_obst, OBSTACLE_SHARE, Some(&zones), 0);
        let mut tiles: Vec<Tile> =
            (0..n * n).map(|i| if blocked[i] { kind_of(zones[i] as usize, kind_noise[i]) } else { Tile::Floor }).collect();
        // fence rings with a one-tile gate: the camp (gate east) and the grove around the boss
        let ring = |centre: (usize, usize), r: usize, gate: (i32, i32), zone: u8, tiles: &mut Vec<Tile>| {
            let (cx, cy) = (centre.0 as i32, centre.1 as i32);
            for dy in -(r as i32)..=r as i32 {
                for dx in -(r as i32)..=r as i32 {
                    if dx.abs().max(dy.abs()) != r as i32 || (dx, dy) == gate {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if x < 0 || y < 0 || x >= n as i32 || y >= n as i32 {
                        continue;
                    }
                    let (x, y) = (x as usize, y as usize);
                    if zones[y * n + x] == zone && !anchors.contains(&(x, y)) {
                        tiles[y * n + x] = Tile::Fence;
                    }
                }
            }
        };
        ring(camp, CAMP_RING, (CAMP_RING as i32, 0), 0, &mut tiles);
        let g = GROVE_RING as i32;
        let gate = [(g, 0), (-g, 0), (0, g), (0, -g)][gate_side as usize];
        ring(anchors[8], GROVE_RING, gate, 8, &mut tiles);
        tiles[camp.1 * n + camp.0] = Tile::Camp;

        let mut m = Map { n, tiles, zones, anchors };
        // reachability: carve an L corridor to any cut-off anchor, then fill unreachable open pockets
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
            m.tiles[y * n + x] = Tile::Floor;
            while (x, y) != b {
                if x != b.0 {
                    x = if x < b.0 { x + 1 } else { x - 1 };
                } else {
                    y = if y < b.1 { y + 1 } else { y - 1 };
                }
                if m.tiles[y * n + x].is_blocking() {
                    m.tiles[y * n + x] = Tile::Floor;
                }
            }
        }
        let seen = m.flood();
        for i in 0..n * n {
            if !seen[i] && !m.tiles[i].is_blocking() {
                m.tiles[i] = kind_of(m.zones[i] as usize, kind_noise[i]);
            }
        }
        // grass on open tiles (never in camp or grove)
        let elig_grass: Vec<bool> = (0..n * n)
            .map(|i| !m.tiles[i].is_blocking() && m.zones[i] != 0 && m.zones[i] != 8 && !m.near(i % n, i / n))
            .collect();
        let grass = top_share(&grass_noise, &elig_grass, GRASS_SHARE, Some(&m.zones), 45);
        for i in 0..n * n {
            if grass[i] {
                m.tiles[i] = Tile::Grass;
            }
        }
        m.tiles[camp.1 * n + camp.0] = Tile::Camp;
        m
    }

    fn near(&self, x: usize, y: usize) -> bool {
        self.anchors.iter().any(|&a| a.0.abs_diff(x) <= 2 && a.1.abs_diff(y) <= 2)
    }

    /// A copy of the map with one tile replaced.
    pub fn with_tile(&self, x: usize, y: usize, t: Tile) -> Map {
        let mut m = Map { n: self.n, tiles: self.tiles.clone(), zones: self.zones.clone(), anchors: self.anchors.clone() };
        m.tiles[y * self.n + x] = t;
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
                if !self.tiles[nb.1 * n + nb.0].is_blocking() && !seen[nb.1 * n + nb.0] {
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

    pub fn size(&self) -> usize {
        self.n
    }
    pub fn tile(&self, x: usize, y: usize) -> Tile {
        self.tiles[y * self.n + x]
    }
    pub fn zone_of(&self, x: usize, y: usize) -> usize {
        self.zones[y * self.n + x] as usize
    }
    pub fn passable(&self, x: usize, y: usize) -> bool {
        x < self.n && y < self.n && self.tiles[y * self.n + x] != Tile::Wall
    }
    pub fn camp(&self) -> (usize, usize) {
        self.anchors[0]
    }
    pub fn anchors(&self) -> &[(usize, usize)] {
        &self.anchors
    }

    /// 4-connected A* (Manhattan heuristic). Ties: lower f, lower h, lower index.
    /// The result excludes `from` and includes `to`; `from == to` gives an empty path.
    pub fn path(&self, from: (usize, usize), to: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        if !self.passable(from.0, from.1) || !self.passable(to.0, to.1) {
            return None;
        }
        if from == to {
            return Some(vec![]);
        }
        let n = self.n;
        let h = |p: (usize, usize)| (p.0.abs_diff(to.0) + p.1.abs_diff(to.1)) as u32;
        let mut g = vec![u32::MAX; n * n];
        let mut prev = vec![usize::MAX; n * n];
        let mut closed = vec![false; n * n];
        let mut heap = BinaryHeap::new();
        let si = from.1 * n + from.0;
        g[si] = 0;
        heap.push(Reverse((h(from), h(from), si)));
        while let Some(Reverse((_, _, i))) = heap.pop() {
            if closed[i] {
                continue;
            }
            closed[i] = true;
            let p = (i % n, i / n);
            if p == to {
                let mut out = vec![];
                let mut c = i;
                while c != si {
                    out.push((c % n, c / n));
                    c = prev[c];
                }
                out.reverse();
                return Some(out);
            }
            for nb in self.nbrs(p) {
                let j = nb.1 * n + nb.0;
                if self.tiles[j] == Tile::Wall || closed[j] {
                    continue;
                }
                let ng = g[i] + 1;
                if ng < g[j] {
                    g[j] = ng;
                    prev[j] = i;
                    heap.push(Reverse((ng + h(nb), h(nb), j)));
                }
            }
        }
        None
    }

    /// The closest passable tile of `zone` by walking distance (BFS). `from` itself if already inside.
    pub fn nearest_tile_of_zone(&self, from: (usize, usize), zone: usize) -> Option<(usize, usize)> {
        if !self.passable(from.0, from.1) {
            return None;
        }
        let n = self.n;
        let mut seen = vec![false; n * n];
        let mut q = VecDeque::new();
        seen[from.1 * n + from.0] = true;
        q.push_back(from);
        while let Some(p) = q.pop_front() {
            if self.zone_of(p.0, p.1) == zone {
                return Some(p);
            }
            for nb in self.nbrs(p) {
                let j = nb.1 * n + nb.0;
                if self.tiles[j] != Tile::Wall && !seen[j] {
                    seen[j] = true;
                    q.push_back(nb);
                }
            }
        }
        None
    }
}

/// JSON body of `GET /api/map`.
pub fn map_body(m: &Map) -> String {
    let n = m.size();
    let mut s = format!("{{\"w\":{},\"h\":{},\"tiles\":[", n, n);
    for y in 0..n {
        if y > 0 {
            s.push(',');
        }
        s.push('"');
        for x in 0..n {
            s.push(match m.tile(x, y) {
                Tile::Floor => '.',
                Tile::Grass => ',',
                Tile::Wall => '#',
                Tile::Camp => 'C',
            });
        }
        s.push('"');
    }
    s.push_str("],\"zones\":[");
    for y in 0..n {
        if y > 0 {
            s.push(',');
        }
        s.push('"');
        for x in 0..n {
            s.push((b'0' + m.zone_of(x, y) as u8) as char);
        }
        s.push('"');
    }
    s.push_str("],\"anchors\":[");
    for (i, a) in m.anchors().iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("[{},{}]", a.0, a.1));
    }
    s.push_str("]}");
    s
}
