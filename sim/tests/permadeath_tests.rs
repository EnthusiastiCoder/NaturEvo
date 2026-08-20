//! Permadeath, outcome and death stats (plan 2026-10-21_hero-map-walking, Addendum 4 part 1). Written before any implementation.
//!
//! API names picked here (the implementer must match or tell the test-writer):
//!   Config.permadeath: bool (default true), settable through Config::set("permadeath", "true|false|1|0").
//!   Run.outcome and Summary.outcome: &'static str, one of "won", "died", "capped".
//!   Run.died_level and Summary.died_level: Option<u32>, hero level at the death event (Some iff a death ended the run).
//!   Event kinds "death" then "end" (final event). Run head JSON "outcome": string.
//!   BatchStats { died: u32, capped: u32, died_pct: f64 (0..100), died_level_hist: Vec<u32>, death_zones: Vec<u32> }
//!   stats JSON keys: "died", "capped", "died_pct", "died_level_hist", "death_zones".
mod common;
use common::parse;
use jungle_sim::config::Config;
use jungle_sim::report::{batch_with, run_body, run_window_body, stats_body};
use jungle_sim::sim::{simulate_summary, simulate_with, Run};

fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = Config::default();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
fn kinds(r: &Run) -> Vec<&'static str> {
    r.events.iter().map(|e| e.kind).collect()
}
