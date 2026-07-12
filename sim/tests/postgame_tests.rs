//! Post-game / event-cap / evolution tests (plan 2026-10-20_jungle-sim-postgame-evolution).
//! Written before the implementation; derived from the plan's API contract.
use jungle_sim::config::{knobs, Config, MAX_EVENT_CAP};
use jungle_sim::data::{stage_name, BOSS, EVOLUTIONS, MAX_STAGE, SPECIES, STAGE_MULT};
use jungle_sim::report::{batch, batch_with, run_body, run_window_body, stats_body};
use jungle_sim::sim::{simulate, simulate_summary, simulate_with, Run};
use std::collections::HashMap;
use std::time::Instant;

fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = Config::default();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
fn stage_of(e: &jungle_sim::sim::Event) -> u8 {
    e.mon.as_ref().map(|m| m.stage).unwrap_or(0)
}
fn fw(r: &Run) -> usize {
    r.first_win_event.expect("expected a win") as usize
}
/// value of a numeric key like `"total":123`
fn num_after(body: &str, key: &str) -> Option<u64> {
    let p = format!("\"{}\":", key);
    let i = body.find(&p)? + p.len();
    let d: String = body[i..].chars().take_while(|c| c.is_ascii_digit()).collect();
    d.parse().ok()
}
fn array_len(body: &str, key: &str) -> usize {
    let p = format!("\"{}\":[", key);
    let i = body.find(&p).unwrap_or_else(|| panic!("missing {}", key)) + p.len();
    let end = body[i..].find(']').unwrap();
    let inner = &body[i..i + end];
    if inner.trim().is_empty() { 0 } else { inner.split(',').count() }
}

// ---------------- config ----------------
#[test]
fn cfg_defaults() {
    let c = Config::default();
    assert_eq!((c.event_cap, c.postgame_level_cap, c.evolve_kills), (65536, 80, 300));
    assert_eq!(MAX_EVENT_CAP, 1_048_576);
}

#[test]
fn cfg_sixteen_knobs_with_ranges() {
    assert_eq!(knobs().len(), 16);
    let find = |k: &str| knobs().iter().find(|x| x.key == k).unwrap_or_else(|| panic!("no knob {}", k));
    for (k, lo, hi, d) in [
        ("event_cap", 100.0, 1_048_576.0, 65536.0),
        ("postgame_level_cap", 40.0, 200.0, 80.0),
        ("evolve_kills", 10.0, 100000.0, 300.0),
    ] {
        let n = find(k);
        assert_eq!((n.min, n.max, n.default), (lo, hi, d), "{}", k);
    }
}

#[test]
fn cfg_set_accepts_bounds() {
    let mut c = Config::default();
    for (k, v) in [
        ("event_cap", "100"), ("event_cap", "1048576"),
        ("postgame_level_cap", "40"), ("postgame_level_cap", "200"),
        ("evolve_kills", "10"), ("evolve_kills", "100000"),
    ] {
        c.set(k, v).unwrap_or_else(|e| panic!("{}={} rejected: {}", k, v, e));
    }
    assert_eq!((c.event_cap, c.postgame_level_cap, c.evolve_kills), (1_048_576, 200, 100000));
    assert!(c.validate().is_ok());
}

#[test]
fn cfg_set_rejects_out_of_range_and_garbage() {
    let mut c = Config::default();
    for (k, v) in [
        ("event_cap", "99"), ("event_cap", "1048577"), ("event_cap", "abc"), ("event_cap", "NaN"),
        ("postgame_level_cap", "39"), ("postgame_level_cap", "201"), ("postgame_level_cap", "abc"), ("postgame_level_cap", "NaN"),
        ("evolve_kills", "9"), ("evolve_kills", "100001"), ("evolve_kills", "abc"), ("evolve_kills", "NaN"),
    ] {
        let e = c.set(k, v).unwrap_err();
        assert!(e.contains(k), "{}={}: {}", k, v, e);
    }
    assert_eq!(c, Config::default());
}

// ---------------- prefix / baselines ----------------
#[test]
fn prefix_first_win_events_and_level() {
    for (seed, f) in [(1u64, 10624u32), (2, 10542), (42, 10616)] {
        let r = simulate(seed);
        assert!(r.won, "seed {}", seed);
        assert_eq!(r.first_win_event, Some(f), "seed {}", seed);
        assert_eq!(r.final_level, 40, "seed {}", seed);
        assert_eq!(r.events[f as usize].kind, "win", "seed {}", seed);
        assert!(r.events[..=f as usize].iter().all(|e| stage_of(e) == 0), "seed {}", seed);
    }
}

#[test]
fn prefix_small_cap_is_prefix_of_default() {
    for seed in [1u64, 7, 42] {
        let full = simulate(seed);
        let short = simulate_with(seed, &cfg(&[("event_cap", "5000")]));
        assert_eq!(short.events.len(), 5000);
        for (a, b) in short.events.iter().zip(full.events.iter()) {
            assert_eq!((a.idx, a.kind, &a.text), (b.idx, b.kind, &b.text), "seed {}", seed);
            assert_eq!((a.hero.level, a.hero.hp, a.hero.gold, a.hero.zone), (b.hero.level, b.hero.hp, b.hero.gold, b.hero.zone));
        }
    }
}

#[test]
fn prefix_across_win_boundary() {
    let full = simulate(1);
    let mid = simulate_with(1, &cfg(&[("event_cap", "20000")]));
    assert_eq!(mid.first_win_event, Some(10624));
    assert_eq!(mid.events.len(), 20000);
    for (a, b) in mid.events.iter().zip(full.events.iter()) {
        assert_eq!((a.kind, &a.text), (b.kind, &b.text));
    }
}

#[test]
fn baseline_win_counts_unchanged() {
    assert_eq!(batch(200, 4).wins, 108);
    assert_eq!(batch(2000, 4).wins, 1076);
}
