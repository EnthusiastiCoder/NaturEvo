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
