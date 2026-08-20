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
