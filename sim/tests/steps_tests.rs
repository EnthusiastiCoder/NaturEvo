//! Per-step walk data (plan 2026-10-21_hero-map-walking, Addendum 3 part B). Written before any implementation.
//!
//! JSON read through run_body / run_window_body / route. Names assumed from the plan:
//!   walk event: "steps": [[x,y],...] (excl. start tile), "nsteps": [[[id,x,y],...],...] one list per hero step.
//!   head "steps" = total hero steps of the run. "From the previous event" means the previous event's hero tile.
//! The 2 MB window budget is taken as 2 * 1024 * 1024 bytes.
mod common;
use common::{parse, J};
use jungle_sim::config::Config;
use jungle_sim::map::Map;
use jungle_sim::report::{run_body, run_window_body};
use jungle_sim::server::route;
use jungle_sim::sim::simulate_with;
use std::collections::HashMap;
use std::path::PathBuf;

type P = (usize, usize);

fn dist(a: P, b: P) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}
fn kind(e: &J) -> &str {
    e.get("kind").unwrap().str()
}
fn hero_xy(e: &J) -> P {
    let h = e.get("hero").unwrap();
    (h.u("x"), h.u("y"))
}
fn events(j: &J) -> &[J] {
    j.get("events").unwrap().arr()
}
fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = Config::default();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
fn steps_of(e: &J) -> Vec<P> {
    e.get("steps").expect("walk event needs steps").arr().iter().map(|p| p.xy()).collect()
}
