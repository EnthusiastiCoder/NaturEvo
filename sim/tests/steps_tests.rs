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

#[test]
fn step_counts_sum_to_the_head_total() {
    let seed = (1u64..=10).find(|s| !simulate_with(*s, &Config::default()).capped).expect("an uncapped seed in 1..=10");
    let j = parse(&run_body(&simulate_with(seed, &Config::default()))).unwrap();
    let sum: usize = events(&j).iter().filter(|e| kind(e) == "walk").map(|e| steps_of(e).len()).sum();
    assert_eq!(sum, j.u("steps"), "seed {}: sum of walk steps vs head steps", seed);
}

#[test]
fn nsteps_matches_steps_and_npcs_move_one_tile() {
    let c = cfg(&[("event_cap", "8000"), ("elites_per_zone", "3")]);
    let j = parse(&run_body(&simulate_with(1, &c))).unwrap();
    let ev = events(&j);
    let mut moves = 0;
    for (i, e) in ev.iter().enumerate() {
        if kind(e) != "walk" {
            continue;
        }
        let st = steps_of(e);
        let ns = e.get("nsteps").expect("walk event needs nsteps").arr();
        assert_eq!(ns.len(), st.len(), "event {}: nsteps length", i);
        // last known tile of each npc: previous event's npcs, then each step's moves
        let mut pos: HashMap<String, P> = HashMap::new();
        if i > 0 {
            for n in ev[i - 1].get("npcs").unwrap().arr() {
                pos.insert(n.get("id").unwrap().str().to_string(), (n.u("x"), n.u("y")));
            }
        }
        let ids: Vec<String> = e.get("npcs").unwrap().arr().iter().map(|n| n.get("id").unwrap().str().to_string()).collect();
        for step in ns {
            for mv in step.arr() {
                let a = mv.arr();
                assert_eq!(a.len(), 3, "event {}: move must be [id,x,y]", i);
                let id = a[0].str().to_string();
                let p = (a[1].num() as usize, a[2].num() as usize);
                assert!(ids.contains(&id), "event {}: unknown npc id {}", i, id);
                if let Some(prev) = pos.get(&id) {
                    assert_eq!(dist(*prev, p), 1, "event {}: npc {} moved {:?}->{:?}", i, id, prev, p);
                }
                pos.insert(id, p);
                moves += 1;
            }
        }
        for n in e.get("npcs").unwrap().arr() {
            let id = n.get("id").unwrap().str().to_string();
            if let Some(p) = pos.get(&id) {
                assert_eq!(*p, (n.u("x"), n.u("y")), "event {}: npc {} end of leg differs from npcs", i, id);
            }
        }
    }
    assert!(moves > 50, "npc moves {}", moves);
}

#[test]
fn same_seed_gives_byte_identical_json_with_steps() {
    let c = cfg(&[("event_cap", "4000")]);
    let a = run_body(&simulate_with(3, &c));
    assert!(a.contains("\"steps\":[["), "no step list in body");
    assert!(a.contains("\"nsteps\":["), "no nsteps in body");
    assert_eq!(a, run_body(&simulate_with(3, &c)));
}

#[test]
fn event_cap_counts_walk_events_not_steps() {
    let c = cfg(&[("event_cap", "300")]);
    let r = simulate_with(1, &c);
    assert_eq!(r.events.len(), 300);
    assert!(r.capped);
    let j = parse(&run_body(&r)).unwrap();
    let ev = events(&j);
    assert!(ev.len() <= 300);
    let total_steps: usize = ev.iter().filter(|e| kind(e) == "walk").map(|e| steps_of(e).len()).sum();
    assert!(total_steps > ev.iter().filter(|e| kind(e) == "walk").count(), "steps must outnumber walk events");
}
