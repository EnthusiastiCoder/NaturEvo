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

#[test]
fn same_seed_same_map_different_seed_differs_only_legend_chars() {
    for size in [32usize, 64] {
        let a = map_body(&Map::generate(5, size));
        assert_eq!(a, map_body(&Map::generate(5, size)));
        assert_ne!(a, map_body(&Map::generate(6, size)));
    }
    for seed in 1u64..=5 {
        let j = parse(&map_body(&Map::generate(seed, 64))).unwrap();
        for row in j.get("tiles").unwrap().arr() {
            assert!(row.str().chars().all(|c| "#RFTW.,C".contains(c)), "seed {} bad char in {}", seed, row.str());
        }
    }
}

#[test]
fn all_five_kinds_appear_each_only_in_allowed_zones() {
    let mut seen: HashSet<String> = HashSet::new();
    for seed in 1u64..=20 {
        let m = Map::generate(seed, 64);
        for (x, y) in all(&m) {
            let t = m.tile(x, y);
            if t.is_blocking() {
                seen.insert(format!("{:?}", t));
                let z = m.zone_of(x, y);
                assert!(allowed(z).contains(&t), "seed {} {:?} at {:?} not allowed in zone {}", seed, t, (x, y), z);
            }
        }
    }
    for k in BLOCKING {
        assert!(seen.contains(&format!("{:?}", k)), "{:?} never appeared in 20 seeds", k);
    }
}

#[test]
fn obstacle_share_is_6_to_18_percent() {
    for size in [32usize, 64, 128] {
        for seed in [1u64, 2, 3, 7] {
            let m = Map::generate(seed, size);
            let b = all(&m).iter().filter(|p| m.tile(p.0, p.1).is_blocking()).count();
            let share = b as f64 / (size * size - 1) as f64;
            assert!((0.06..=0.18).contains(&share), "size {} seed {} share {}", size, seed, share);
        }
    }
}

#[test]
fn connectivity_and_placement_rules_hold_over_50_seeds() {
    for seed in 1u64..=50 {
        let m = Map::generate(seed, 64);
        let seen = flood(&m);
        let boss = m.anchors()[8];
        let c = m.camp();
        assert_eq!(m.tile(c.0, c.1), Tile::Camp);
        assert!(!m.tile(boss.0, boss.1).is_blocking(), "seed {} boss tile blocked", seed);
        assert!(seen.contains(&boss), "seed {} boss unreachable", seed);
        let mut grass = [0usize; 9];
        for (x, y) in all(&m) {
            let t = m.tile(x, y);
            assert_eq!(m.passable(x, y), !t.is_blocking(), "seed {} passable vs tile at {:?}", seed, (x, y));
            if t.is_blocking() {
                continue;
            }
            assert!(seen.contains(&(x, y)), "seed {} open tile {:?} unreachable from camp", seed, (x, y));
            if t == Tile::Grass {
                grass[m.zone_of(x, y)] += 1;
            }
        }
        for z in 1..=7 {
            assert!(grass[z] >= 20, "seed {} zone {} has {} grass", seed, z, grass[z]);
        }
        for (z, a) in m.anchors().iter().enumerate() {
            assert!(!m.tile(a.0, a.1).is_blocking(), "seed {} anchor {} blocked", seed, z);
            if z >= 1 {
                assert!(m.path(c, *a).is_some(), "seed {} no A* path camp -> zone {} anchor", seed, z);
            }
        }
        assert!(m.path(c, boss).is_some(), "seed {} no A* path to boss", seed);
        // gates: the camp zone is a fence ring that still lets the hero out
        let ring = all(&m).iter().filter(|p| m.zone_of(p.0, p.1) == 0 && m.tile(p.0, p.1) == Tile::Fence).count();
        assert!(ring >= 4, "seed {} camp zone has {} fence tiles, expected a ring", seed, ring);
        assert!(m.path(c, m.anchors()[1]).is_some(), "seed {} camp ring has no gate", seed);
    }
}

#[test]
fn astar_never_crosses_blocking_and_detours_around_a_hand_built_block() {
    let m = Map::generate(3, 64);
    for z in 1..9 {
        for &(x, y) in &m.path(m.camp(), m.anchors()[z]).unwrap() {
            assert!(!m.tile(x, y).is_blocking(), "path crosses blocking tile at {:?}", (x, y));
        }
    }
    // find a 7-wide, 3-tall fully open window and block the middle of the centre row
    let n = m.size();
    let open = |x: usize, y: usize| !m.tile(x, y).is_blocking();
    let mut found = None;
    'f: for y in 1..n - 1 {
        for x in 0..n - 6 {
            if (y - 1..=y + 1).all(|yy| (x..x + 7).all(|xx| open(xx, yy))) {
                found = Some((x, y));
                break 'f;
            }
        }
    }
    let (x, y) = found.expect("an open 7x3 window");
    let (a, b) = ((x, y), (x + 6, y));
    assert_eq!(m.path(a, b).unwrap().len(), 6);
    let blocked = m.with_tile(x + 3, y, Tile::Rock);
    assert_eq!(blocked.tile(x + 3, y), Tile::Rock);
    assert!(!blocked.passable(x + 3, y));
    let p = blocked.path(a, b).expect("detour exists");
    assert!(!p.contains(&(x + 3, y)), "path goes through the block");
    assert_eq!(p.len(), 8, "detour around one tile is 2 steps longer");
    assert_eq!(*p.last().unwrap(), b);
    assert_eq!(blocked.path(a, (x + 3, y)), None, "blocked target");
}

#[test]
fn no_entity_stands_on_a_blocking_tile_in_5_runs() {
    for seed in 1u64..=5 {
        let c = cfg(&[("event_cap", "8000"), ("elites_per_zone", "3")]);
        let m = Map::generate(seed, c.map_size as usize);
        let j = parse(&run_body(&simulate_with(seed, &c))).unwrap();
        let mut seen = 0;
        for e in j.get("events").unwrap().arr() {
            let h = e.get("hero").unwrap();
            let mut pts = vec![(h.u("x"), h.u("y"))];
            if let Some(mo) = e.get("mon").filter(|m| !m.is_null()) {
                pts.push((mo.u("x"), mo.u("y")));
            }
            for n in e.get("npcs").unwrap().arr() {
                pts.push((n.u("x"), n.u("y")));
            }
            for p in pts {
                seen += 1;
                assert!(m.passable(p.0, p.1), "seed {} entity on blocking tile {:?} ({:?})", seed, p, m.tile(p.0, p.1));
            }
        }
        assert!(seen > 500);
    }
}

#[test]
fn encounters_still_fire_at_the_configured_rate() {
    let rate = 0.2f64;
    let (mut steps, mut enc) = (0f64, 0f64);
    for seed in 1u64..=10 {
        let c = cfg(&[("encounter_rate", "0.2"), ("event_cap", "4000"), ("elites_per_zone", "0")]);
        let r = simulate_with(seed, &c);
        steps += r.steps as f64;
        enc += r.events.iter().filter(|e| e.kind == "encounter").count() as f64;
    }
    assert!(enc > 50.0, "encounters {}", enc);
    // a 6-step cooldown follows every fight (Addendum 4); the rate acts on the steps after it,
    // and only grass tiles roll (at least a third of the steps on this map)
    let per = steps / enc - 6.0;
    assert!(per >= 0.5 / rate && per <= 4.0 / rate, "steps per encounter {} not within a factor 2 of {}", per, 1.0 / rate);
}

#[test]
fn api_map_legend_has_8_entries_and_tiles_use_only_legend_chars() {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../viewer"));
    let r = route(&root, "GET", "/api/map?seed=4&map_size=48", Some("127.0.0.1:8080"));
    assert_eq!(r.status, 200);
    let j = parse(&String::from_utf8(r.body).unwrap()).unwrap();
    let legend = j.get("legend").expect("legend field").arr();
    assert_eq!(legend.len(), 8);
    let want = [('#', "Wall", true), ('R', "Rock", true), ('F', "Fence", true), ('T', "Tree", true), ('W', "Water", true), ('.', "Floor", false), (',', "Grass", false), ('C', "Camp", false)];
    let mut chars = String::new();
    for (e, (ch, name, blocking)) in legend.iter().zip(want) {
        assert_eq!(e.get("ch").unwrap().str(), ch.to_string());
        assert_eq!(e.get("name").unwrap().str(), name);
        assert_eq!(e.get("blocking"), Some(&common::J::Bool(blocking)), "blocking flag of {}", name);
        chars.push(ch);
    }
    for row in j.get("tiles").unwrap().arr() {
        assert!(row.str().chars().all(|c| chars.contains(c)), "tile char outside the legend in {}", row.str());
    }
    for k in ["w", "h", "tiles", "zones", "anchors"] {
        assert!(j.get(k).is_some(), "old field {} missing", k);
    }
}
