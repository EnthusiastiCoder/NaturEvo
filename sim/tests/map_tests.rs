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

type P = (usize, usize);

fn all(m: &Map) -> impl Iterator<Item = P> {
    let n = m.size();
    (0..n).flat_map(move |y| (0..n).map(move |x| (x, y)))
}
fn nbrs(m: &Map, (x, y): P) -> Vec<P> {
    let n = m.size();
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
fn reachable(m: &Map) -> Vec<Vec<bool>> {
    let n = m.size();
    let mut seen = vec![vec![false; n]; n];
    let c = m.camp();
    let mut q = VecDeque::new();
    seen[c.1][c.0] = true;
    q.push_back(c);
    while let Some(p) = q.pop_front() {
        for nb in nbrs(m, p) {
            if m.passable(nb.0, nb.1) && !seen[nb.1][nb.0] {
                seen[nb.1][nb.0] = true;
                q.push_back(nb);
            }
        }
    }
    seen
}
fn manhattan(a: P, b: P) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}
fn check_map(m: &Map, label: &str) {
    let n = m.size();
    let total = (n * n) as f64;
    // zones in range, camp, no grass in camp zone and grove
    let mut walls = 0usize;
    let mut grass = [0usize; 9];
    let mut floor_in_8 = 0usize;
    let seen = reachable(m);
    for (x, y) in all(m) {
        let z = m.zone_of(x, y);
        assert!(z <= 8, "{} zone {} at {:?}", label, z, (x, y));
        let t = m.tile(x, y);
        assert_eq!(m.passable(x, y), t != Tile::Wall, "{} passable vs tile at {:?}", label, (x, y));
        if t == Tile::Wall {
            walls += 1;
            continue;
        }
        assert!(seen[y][x], "{} non-wall tile {:?} unreachable from camp", label, (x, y));
        if t == Tile::Grass {
            grass[z] += 1;
        }
        if z == 8 && t == Tile::Floor {
            floor_in_8 += 1;
        }
    }
    let share = walls as f64 / total;
    assert!((0.03..=0.20).contains(&share), "{} wall share {}", label, share);
    assert_eq!(grass[0], 0, "{} camp zone has grass", label);
    assert_eq!(grass[8], 0, "{} grove has grass", label);
    for z in 1..=7 {
        assert!(grass[z] >= 20, "{} zone {} has {} grass tiles", label, z, grass[z]);
    }
    assert!(floor_in_8 >= 1, "{} grove has no reachable floor", label);
    let c = m.camp();
    assert_eq!(m.tile(c.0, c.1), Tile::Camp, "{} camp tile kind", label);
    assert_eq!(m.zone_of(c.0, c.1), 0, "{} camp zone", label);
    assert_eq!(m.anchors().len(), 9);
    for (z, a) in m.anchors().iter().enumerate() {
        assert!(m.passable(a.0, a.1), "{} anchor {} on a wall", label, z);
        assert_eq!(m.zone_of(a.0, a.1), z, "{} anchor {} zone", label, z);
    }
    // 4-connected zones (walls count as part of their zone)
    for z in 0..9 {
        let tiles: Vec<P> = all(m).filter(|&(x, y)| m.zone_of(x, y) == z).collect();
        assert!(!tiles.is_empty(), "{} zone {} empty", label, z);
        let mut vis = std::collections::HashSet::new();
        let mut q = VecDeque::new();
        vis.insert(tiles[0]);
        q.push_back(tiles[0]);
        while let Some(p) = q.pop_front() {
            for nb in nbrs(m, p) {
                if m.zone_of(nb.0, nb.1) == z && vis.insert(nb) {
                    q.push_back(nb);
                }
            }
        }
        assert_eq!(vis.len(), tiles.len(), "{} zone {} is not 4-connected", label, z);
    }
}

#[test]
fn same_seed_same_map_different_seed_differs() {
    for size in [32usize, 64] {
        let a = map_body(&Map::generate(5, size));
        let b = map_body(&Map::generate(5, size));
        assert_eq!(a, b);
        assert_ne!(a, map_body(&Map::generate(6, size)));
    }
}

#[test]
fn map_invariants_hold_at_sizes_32_64_128() {
    for size in [32usize, 64, 128] {
        for seed in [1u64, 2, 3, 99] {
            let m = Map::generate(seed, size);
            assert_eq!(m.size(), size);
            check_map(&m, &format!("size {} seed {}", size, seed));
        }
    }
}

#[test]
fn out_of_bounds_is_not_passable() {
    let m = Map::generate(1, 32);
    assert!(!m.passable(32, 0));
    assert!(!m.passable(0, 32));
    assert!(!m.passable(1000, 1000));
}

#[test]
fn map_body_schema() {
    let m = Map::generate(1, 32);
    let b = map_body(&m);
    assert!(b.starts_with("{\"w\":32,\"h\":32,") || (b.contains("\"w\":32") && b.contains("\"h\":32")), "{}", &b[..b.len().min(80)]);
    for k in ["\"tiles\":[", "\"zones\":[", "\"anchors\":[["] {
        assert!(b.contains(k), "missing {}", k);
    }
}

#[test]
fn astar_open_path_is_manhattan_and_valid() {
    let m = Map::generate(7, 64);
    let n = m.size();
    let mut open_pairs = 0;
    let mut tested = 0;
    for ay in (0..n).step_by(5) {
        for ax in (0..n).step_by(7) {
            for (bx, by) in [(ax + 6, ay + 4), (ax + 9, ay), (ax, ay + 11)] {
                if bx >= n || by >= n || !m.passable(ax, ay) || !m.passable(bx, by) {
                    continue;
                }
                let (a, b) = ((ax, ay), (bx, by));
                let p = m.path(a, b).unwrap_or_else(|| panic!("no path {:?}->{:?}", a, b));
                tested += 1;
                // valid: unit steps, passable, no repeats, ends at goal, never the start
                let mut prev = a;
                let mut seen = std::collections::HashSet::new();
                seen.insert(a);
                for &t in &p {
                    assert_eq!(manhattan(prev, t), 1, "non-unit step {:?}->{:?}", prev, t);
                    assert!(m.passable(t.0, t.1), "path crosses wall at {:?}", t);
                    assert!(seen.insert(t), "path repeats tile {:?}", t);
                    prev = t;
                }
                assert_eq!(*p.last().unwrap(), b);
                assert!(p.len() >= manhattan(a, b));
                assert_eq!(m.path(a, b), Some(p.clone()), "A* not deterministic");
                // open rectangle: every tile in the bounding box is passable => optimal is Manhattan
                let (x0, x1) = (ax.min(bx), ax.max(bx));
                let (y0, y1) = (ay.min(by), ay.max(by));
                let clear = (y0..=y1).all(|y| (x0..=x1).all(|x| m.passable(x, y)));
                if clear {
                    open_pairs += 1;
                    assert_eq!(p.len(), manhattan(a, b), "open path {:?}->{:?} not Manhattan", a, b);
                }
            }
        }
    }
    assert!(tested > 50, "too few pairs tested: {}", tested);
    assert!(open_pairs >= 5, "too few open-rectangle pairs: {}", open_pairs);
}

#[test]
fn astar_edge_cases() {
    let m = Map::generate(3, 64);
    let c = m.camp();
    assert_eq!(m.path(c, c), Some(vec![]));
    let wall = all(&m).find(|&(x, y)| m.tile(x, y) == Tile::Wall).expect("map has walls");
    assert_eq!(m.path(c, wall), None, "wall target");
    assert_eq!(m.path(c, (64, 64)), None, "out of bounds target");
    assert_eq!(m.path(c, (10_000, 3)), None);
}

#[test]
fn astar_reaches_every_zone_anchor_from_camp() {
    let m = Map::generate(11, 64);
    for z in 1..9 {
        let a = m.anchors()[z];
        let p = m.path(m.camp(), a).unwrap_or_else(|| panic!("zone {} unreachable", z));
        assert!(!p.is_empty());
    }
}

#[test]
fn nearest_tile_of_zone_is_in_zone_and_nearest() {
    let m = Map::generate(4, 64);
    let from = m.camp();
    for z in 1..9 {
        let t = m.nearest_tile_of_zone(from, z).unwrap_or_else(|| panic!("zone {}", z));
        assert_eq!(m.zone_of(t.0, t.1), z);
        assert!(m.passable(t.0, t.1));
        let best = m.path(from, t).unwrap().len();
        for q in all(&m).filter(|&(x, y)| m.zone_of(x, y) == z && m.passable(x, y)) {
            let l = m.path(from, q).unwrap().len();
            assert!(best <= l, "zone {}: {:?} (len {}) nearer than {:?} (len {})", z, q, l, t, best);
        }
    }
    // already in the zone: returns a tile at distance 0 (itself)
    let a = m.anchors()[3];
    assert_eq!(m.nearest_tile_of_zone(a, 3), Some(a));
}

#[test]
fn zone_species_weights_follow_the_rule() {
    for (i, z) in ZONES.iter().enumerate() {
        assert!(z.species.iter().map(|s| s.1).sum::<u32>() > 0, "zone {} weights", i);
        for s in z.species {
            assert!(s.0 < SPECIES.len() && s.1 > 0, "zone {} entry {:?}", i, s);
        }
    }
    // zone 1..7 have 3 or 4 species; weights 4/3/2/1 (4 species) or 4/3/1 (3 species)
    for i in 1..=7 {
        let w: Vec<u32> = ZONES[i].species.iter().map(|s| s.1).collect();
        match w.len() {
            4 => assert_eq!(w, vec![4, 3, 2, 1], "zone {}", i),
            3 => assert_eq!(w, vec![4, 3, 1], "zone {}", i),
            n => panic!("zone {} has {} species", i, n),
        }
    }
    assert_eq!(ZONES[8].species.len(), 1, "boss zone has only the boss");
}
