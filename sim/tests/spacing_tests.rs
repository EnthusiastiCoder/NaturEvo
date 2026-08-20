//! NPC and hero spacing, roaming limits and encounter pacing (plan Addendum 4 part 2). Written before any implementation.
//! Homes are read as each NPC's position in the first event (events[0].npcs); hero start is events[0].hero.
//! Positions over time are rebuilt from the event npcs list plus the per-step nsteps moves.
//! Scaled minimum = max(6, base * size / 64). Zone anchors pairwise >= scaled 14.
mod common;
use common::{parse, J};
use jungle_sim::config::Config;
use jungle_sim::map::Map;
use jungle_sim::report::run_body;
use jungle_sim::sim::simulate_with;
use std::collections::HashMap;

type P = (usize, usize);
fn dist(a: P, b: P) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}
fn scaled(base: usize, size: usize) -> usize {
    (base * size / 64).max(6)
}
fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = Config::default();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
fn homes(j: &J) -> Vec<(String, P)> {
    let ev = j.get("events").unwrap().arr();
    ev[0].get("npcs").unwrap().arr().iter().map(|n| (n.get("id").unwrap().str().to_string(), (n.u("x"), n.u("y")))).collect()
}
