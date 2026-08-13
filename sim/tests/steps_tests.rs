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

#[test]
fn step_lists_are_unit_moves_on_open_tiles_ending_at_the_hero() {
    for seed in [1u64, 2] {
        let c = cfg(&[("event_cap", "8000")]);
        let m = Map::generate(seed, c.map_size as usize);
        let j = parse(&run_body(&simulate_with(seed, &c))).unwrap();
        let ev = events(&j);
        let mut walks = 0;
        for (i, e) in ev.iter().enumerate() {
            if kind(e) != "walk" {
                assert!(e.get("steps").is_none() && e.get("nsteps").is_none(), "{} event {} carries steps", kind(e), i);
                continue;
            }
            walks += 1;
            let st = steps_of(e);
            assert!(!st.is_empty() && st.len() <= 256, "seed {} event {} has {} steps", seed, i, st.len());
            assert_eq!(*st.last().unwrap(), hero_xy(e), "event {}: last step must equal hero tile", i);
            if i > 0 && kind(&ev[i - 1]) != "travel" {
                assert_eq!(dist(st[0], hero_xy(&ev[i - 1])), 1, "event {}: first step not adjacent to previous hero tile", i);
            }
            for w in st.windows(2) {
                assert_eq!(dist(w[0], w[1]), 1, "event {}: steps {:?}->{:?} not one tile in one axis", i, w[0], w[1]);
            }
            for t in &st {
                assert!(m.passable(t.0, t.1), "event {}: step on blocking tile {:?}", i, t);
            }
            // path waypoints stay for compatibility
            assert!(e.get("path").is_some(), "event {}: path must remain", i);
        }
        assert!(walks > 20, "walks {}", walks);
    }
}
