//! Walking hero, encounters, elites, moves, determinism and cap tests
//! (plan 2026-10-21_hero-map-walking, workstream 2). Written from the plan before any implementation.
//!
//! Events are read through run_body() JSON (parsed by tests/common) so only these Rust names are assumed:
//!   Run.steps (counter), Summary.steps, Config fields from the plan, jungle_sim::map::Map (see map_tests.rs).
//! The map of a run is assumed to be Map::generate(seed, cfg.map_size).
mod common;
use common::{parse, J};
use jungle_sim::config::Config;
use jungle_sim::data::{stage_name, MAX_STAGE, MOVES, SPECIES, ZONES};
use jungle_sim::map::{Map, Tile};
use jungle_sim::report::{batch_with, run_body, run_window_body};
use jungle_sim::sim::{simulate_summary, simulate_with, Run};
use std::collections::{HashMap, HashSet};

fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = Config::default();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
fn dist(a: (usize, usize), b: (usize, usize)) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}
fn hero_xy(e: &J) -> (usize, usize) {
    let h = e.get("hero").expect("hero");
    (h.u("x"), h.u("y"))
}
fn kind(e: &J) -> &str {
    e.get("kind").unwrap().str()
}
fn text(e: &J) -> &str {
    e.get("text").unwrap().str()
}
fn mon(e: &J) -> Option<&J> {
    e.get("mon").filter(|m| !m.is_null())
}
fn load(seed: u64, c: &Config) -> (Run, J, Map) {
    let run = simulate_with(seed, c);
    let j = parse(&run_body(&run)).expect("run_body must be valid JSON");
    let m = Map::generate(seed, c.map_size as usize);
    (run, j, m)
}
fn events(j: &J) -> &[J] {
    j.get("events").unwrap().arr()
}
fn won_seed() -> u64 {
    (1u64..=40).find(|s| simulate_with(*s, &Config::default()).won).expect("some seed in 1..=40 must win")
}
/// expand waypoints into unit steps; waypoints must be axis-aligned
fn expand(path: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut out = vec![path[0]];
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert!(a.0 == b.0 || a.1 == b.1, "waypoints {:?}->{:?} differ in both axes", a, b);
        let (mut x, mut y) = a;
        while (x, y) != b {
            if x < b.0 {
                x += 1
            } else if x > b.0 {
                x -= 1
            } else if y < b.1 {
                y += 1
            } else {
                y -= 1
            }
            out.push((x, y));
        }
    }
    out
}

#[test]
fn hero_positions_are_in_map_and_on_floor() {
    for seed in [1u64, 2, 3] {
        let (_, j, m) = load(seed, &cfg(&[("event_cap", "6000")]));
        for e in events(&j) {
            let p = hero_xy(e);
            assert!(p.0 < m.size() && p.1 < m.size(), "seed {} event out of map {:?}", seed, p);
            assert!(m.passable(p.0, p.1), "seed {} hero on a wall at {:?}", seed, p);
            assert_eq!(m.zone_of(p.0, p.1), e.get("hero").unwrap().u("zone"), "hero.zone must be the zone of the hero tile");
        }
    }
}

#[test]
fn walk_events_have_valid_paths() {
    let (_, j, m) = load(1, &cfg(&[("event_cap", "8000")]));
    let ev = events(&j);
    let mut walks = 0;
    for (i, e) in ev.iter().enumerate() {
        if kind(e) != "walk" {
            continue;
        }
        walks += 1;
        let path: Vec<(usize, usize)> = e.get("path").expect("walk event needs path").arr().iter().map(|p| p.xy()).collect();
        assert!(!path.is_empty() && path.len() <= 64, "event {} path len {}", i, path.len());
        assert_eq!(*path.last().unwrap(), hero_xy(e), "event {}: path must end at the hero tile", i);
        if i > 0 && kind(&ev[i - 1]) != "travel" {
            assert!(dist(path[0], hero_xy(&ev[i - 1])) <= 1, "event {}: path must start adjacent to the previous hero tile", i);
        }
        let tiles = expand(&path);
        for w in tiles.windows(2) {
            assert!(dist(w[0], w[1]) <= 1, "event {}: tiles {:?}->{:?} more than 1 apart", i, w[0], w[1]);
        }
        for t in &tiles {
            assert!(m.passable(t.0, t.1), "event {}: path crosses a wall at {:?}", i, t);
        }
    }
    assert!(walks > 20, "expected many walk events, got {}", walks);
}

#[test]
fn only_walk_events_carry_a_path() {
    let (_, j, _) = load(1, &cfg(&[("event_cap", "3000")]));
    for e in events(&j) {
        if kind(e) != "walk" {
            assert!(e.get("path").is_none(), "{} event has a path", kind(e));
        }
    }
}

#[test]
fn encounters_happen_on_the_hero_tile_and_on_grass() {
    let (_, j, m) = load(2, &cfg(&[("event_cap", "8000")]));
    let mut n = 0;
    for e in events(&j).iter().filter(|e| kind(e) == "encounter") {
        n += 1;
        let mo = mon(e).expect("encounter has a mon");
        let p = hero_xy(e);
        assert_eq!((mo.u("x"), mo.u("y")), p, "encounter mon must stand on the hero tile");
        assert_eq!(m.tile(p.0, p.1), Tile::Grass, "encounter at {:?} is not Grass", p);
    }
    assert!(n > 50, "encounters: {}", n);
}

#[test]
fn elite_fights_are_adjacent_and_use_zone_species() {
    let mut n = 0;
    for seed in 1u64..=6 {
        let (_, j, _) = load(seed, &cfg(&[("event_cap", "20000"), ("elites_per_zone", "3")]));
        for e in events(&j).iter().filter(|e| kind(e) == "elite") {
            n += 1;
            let mo = mon(e).expect("elite has a mon");
            assert!(dist((mo.u("x"), mo.u("y")), hero_xy(e)) <= 1, "elite not adjacent to hero");
            let z = e.get("hero").unwrap().u("zone");
            assert!(ZONES[z].species.iter().any(|s| s.0 == mo.u("species")), "elite species not in zone {} table", z);
        }
    }
    assert!(n > 0, "no elite fights observed");
}

#[test]
fn elites_disabled_gives_no_elite_events() {
    let (_, j, _) = load(1, &cfg(&[("event_cap", "8000"), ("elites_per_zone", "0")]));
    assert!(events(&j).iter().all(|e| kind(e) != "elite"));
    for e in events(&j) {
        for n in e.get("npcs").unwrap().arr() {
            assert!(!n.get("id").unwrap().str().starts_with('e'), "elite npc present with elites_per_zone=0");
        }
    }
}

#[test]
fn hero_starts_at_camp_and_camp_events_happen_there() {
    let c = Config::default();
    let (_, j, m) = load(1, &cfg(&[("event_cap", "20000")]));
    let camp = m.camp();
    let ev = events(&j);
    assert_eq!(hero_xy(&ev[0]), camp, "hero starts on the camp tile");
    let (mut respawns, mut rests) = (0, 0);
    for (i, e) in ev.iter().enumerate() {
        match kind(e) {
            "respawn" => {
                respawns += 1;
                assert_eq!(hero_xy(e), camp, "respawn {} not at camp", i);
            }
            "shop" => assert_eq!(hero_xy(e), camp, "shop event {} away from camp", i),
            "rest" if mon(e).is_none() => {
                rests += 1;
                assert_eq!(hero_xy(e), camp, "rest event {} away from camp", i)
            }
            _ => {}
        }
    }
    assert!(respawns > 0 && rests > 0, "respawns {} rests {}", respawns, rests);
    let _ = c;
}

#[test]
fn won_run_reaches_the_grove_and_fights_boss_near_the_boss_npc() {
    let seed = won_seed();
    let (run, j, m) = load(seed, &Config::default());
    assert!(run.won);
    let ev = events(&j);
    assert!(ev.iter().any(|e| m.zone_of(hero_xy(e).0, hero_xy(e).1) == 8), "hero never reached zone 8");
    let mut boss_fights = 0;
    for e in ev.iter().filter(|e| kind(e) == "boss") {
        boss_fights += 1;
        let npc = e.get("npcs").unwrap().arr().iter().find(|n| n.get("id").unwrap().str() == "boss").expect("boss npc listed");
        assert!(dist(hero_xy(e), (npc.u("x"), npc.u("y"))) <= 2, "boss fight farther than 2 tiles from the boss NPC");
    }
    assert!(boss_fights > 0);
}

#[test]
fn npcs_are_valid_and_in_the_hero_zone() {
    let (_, j, m) = load(1, &cfg(&[("event_cap", "10000"), ("elites_per_zone", "3")]));
    let mut seen = 0;
    for e in events(&j) {
        let npcs = e.get("npcs").expect("npcs array on every event").arr();
        assert!(npcs.len() <= 3, "{} npcs", npcs.len());
        let z = e.get("hero").unwrap().u("zone");
        for n in npcs {
            seen += 1;
            let (x, y) = (n.u("x"), n.u("y"));
            assert!(x < m.size() && y < m.size() && m.passable(x, y), "npc {:?} off map or on wall", n);
            assert_eq!(m.zone_of(x, y), z, "npc outside the hero zone");
            assert!(!n.get("id").unwrap().str().is_empty());
            assert!(n.get("sp").is_some() && n.get("lv").is_some());
        }
    }
    assert!(seen > 100, "npcs seen: {}", seen);
}

#[test]
fn encounter_rate_scales_steps_per_encounter() {
    let avg = |rate: &str| {
        let (mut steps, mut enc) = (0f64, 0f64);
        for seed in 1u64..=10 {
            let c = cfg(&[("encounter_rate", rate), ("event_cap", "4000"), ("elites_per_zone", "0")]);
            let r = simulate_with(seed, &c);
            steps += r.steps as f64;
            enc += r.events.iter().filter(|e| e.kind == "encounter").count() as f64;
        }
        steps / enc
    };
    let ratio = avg("0.05") / avg("0.5");
    assert!((5.0..=20.0).contains(&ratio), "steps/encounter ratio {}", ratio);
}

#[test]
fn steps_are_not_events() {
    let r = simulate_with(1, &Config::default());
    assert!(r.events.len() <= 65536);
    assert!(r.steps as usize > r.events.len(), "steps {} events {}", r.steps, r.events.len());
    let j = parse(&run_window_body(&r, 0, 10)).unwrap();
    assert_eq!(j.u("steps") as u64, r.steps as u64);
}

#[test]
fn max_steps_terminates_with_capped() {
    let c = cfg(&[("encounter_rate", "0.01"), ("max_steps", "1000")]);
    let r = simulate_with(1, &c);
    assert!(r.capped, "max_steps must set capped");
    assert!(r.steps as u64 <= 1000, "steps {}", r.steps);
    assert!(r.events.len() < 65536);
    let s = simulate_summary(1, &c);
    assert!(s.capped);
    assert_eq!(s.steps as u64, r.steps as u64);
}
