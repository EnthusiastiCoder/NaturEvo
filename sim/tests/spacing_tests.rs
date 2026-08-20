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

#[test]
fn home_minimums_hold_at_sizes_32_64_128() {
    for size in [32usize, 64, 128] {
        for seed in 1..=50u64 {
            let c = cfg(&[("map_size", &size.to_string()), ("event_cap", "100")]);
            let m = Map::generate(seed, size);
            let j = parse(&run_body(&simulate_with(seed, &c))).unwrap();
            let h = homes(&j);
            assert!(h.len() >= 2, "seed {} size {}: npcs {}", seed, size, h.len());
            let hero = j.get("events").unwrap().arr()[0].get("hero").unwrap();
            assert_eq!((hero.u("x"), hero.u("y")), m.camp(), "seed {} size {}: hero must start on camp", seed, size);
            for (id, p) in &h {
                let min = if id == "boss" { scaled(20, size) } else { scaled(12, size) };
                assert!(dist(*p, m.camp()) >= min, "seed {} size {}: {} home {:?} only {} from camp (min {})", seed, size, id, p, dist(*p, m.camp()), min);
                assert!(m.passable(p.0, p.1), "seed {} size {}: {} home on blocking tile", seed, size, id);
            }
            for i in 0..h.len() {
                for k in i + 1..h.len() {
                    assert!(dist(h[i].1, h[k].1) >= scaled(14, size), "seed {} size {}: {} and {} homes {} apart", seed, size, h[i].0, h[k].0, dist(h[i].1, h[k].1));
                }
            }
        }
    }
}

#[test]
fn zone_anchors_are_spread() {
    for size in [32usize, 64, 128] {
        for seed in 1..=50u64 {
            let m = Map::generate(seed, size);
            let a = m.anchors();
            for i in 0..a.len() {
                for k in i + 1..a.len() {
                    assert!(dist(a[i], a[k]) >= scaled(14, size), "seed {} size {}: anchors {:?} {:?}", seed, size, a[i], a[k]);
                }
            }
        }
    }
}

#[test]
fn npcs_stay_in_radius_zone_open_tiles_and_apart() {
    for seed in 1..=8u64 {
        let c = cfg(&[("event_cap", "4000"), ("permadeath", "false")]);
        let m = Map::generate(seed, c.map_size as usize);
        let j = parse(&run_body(&simulate_with(seed, &c))).unwrap();
        let home: HashMap<String, P> = homes(&j).into_iter().collect();
        let mut cur: HashMap<String, P> = home.clone();
        let check = |cur: &HashMap<String, P>, ctx: &str| {
            let ids: Vec<&String> = cur.keys().collect();
            for id in &ids {
                let p = cur[*id];
                let h = home[*id];
                let max = if *id == "boss" { 2 } else { 6 };
                assert!(dist(p, h) <= max, "seed {} {}: {} at {:?} is {} from home {:?} (max {})", seed, ctx, id, p, dist(p, h), h, max);
                assert!(m.passable(p.0, p.1), "seed {} {}: {} on blocking tile {:?}", seed, ctx, id, p);
                if *id != "boss" {
                    assert_eq!(m.zone_of(p.0, p.1), m.zone_of(h.0, h.1), "seed {} {}: {} left its zone", seed, ctx, id);
                }
            }
            for i in 0..ids.len() {
                for k in i + 1..ids.len() {
                    assert!(dist(cur[ids[i]], cur[ids[k]]) >= 4, "seed {} {}: {} and {} only {} apart", seed, ctx, ids[i], ids[k], dist(cur[ids[i]], cur[ids[k]]));
                }
            }
        };
        for (n, e) in j.get("events").unwrap().arr().iter().enumerate() {
            if let Some(ns) = e.get("nsteps") {
                for mv in ns.arr() {
                    for t in mv.arr() {
                        let a = t.arr();
                        cur.insert(a[0].str().to_string(), (a[1].num() as usize, a[2].num() as usize));
                    }
                    check(&cur, &format!("step in event {}", n));
                }
            }
            for t in e.get("npcs").unwrap().arr() {
                cur.insert(t.get("id").unwrap().str().to_string(), (t.u("x"), t.u("y")));
            }
            check(&cur, &format!("event {}", n));
        }
    }
}

#[test]
fn fights_are_spaced_out() {
    let mut gaps: Vec<usize> = Vec::new();
    for seed in 1..=10u64 {
        let c = cfg(&[("event_cap", "6000"), ("permadeath", "false")]);
        let j = parse(&run_body(&simulate_with(seed, &c))).unwrap();
        let mut step = 0usize;
        let mut in_fight = false;
        let mut last: Option<usize> = None;
        for e in j.get("events").unwrap().arr() {
            if e.get("kind").unwrap().str() == "walk" {
                step += e.get("steps").expect("steps").arr().len();
            }
            let fighting = !e.get("mon").unwrap().is_null();
            if fighting && !in_fight {
                if let Some(l) = last {
                    assert!(step - l >= 6, "seed {}: fights {} steps apart", seed, step - l);
                    gaps.push(step - l);
                }
                last = Some(step);
            }
            in_fight = fighting;
        }
    }
    assert!(gaps.len() > 30, "only {} gaps", gaps.len());
    let avg = gaps.iter().sum::<usize>() as f64 / gaps.len() as f64;
    assert!(avg >= 10.0 && avg <= 25.0, "average steps between fights {}", avg);
}
