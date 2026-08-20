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

#[test]
fn on_death_is_followed_only_by_end() {
    let c = cfg(&[("permadeath", "true")]);
    let mut died = 0;
    let mut survived = 0;
    for seed in 0..200u64 {
        let r = simulate_with(seed, &c);
        let k = kinds(&r);
        let deaths = k.iter().filter(|x| **x == "death").count();
        assert!(deaths <= 1, "seed {} has {} death events", seed, deaths);
        assert!(r.deaths <= 1, "seed {} deaths {}", seed, r.deaths);
        assert!(!k.contains(&"respawn"), "seed {} respawned under permadeath", seed);
        if deaths == 1 {
            died += 1;
            let n = k.len();
            assert!(n >= 2 && k[n - 2] == "death" && k[n - 1] == "end", "seed {}: tail {:?}", seed, &k[n.saturating_sub(3)..]);
            assert_eq!(k.iter().filter(|x| **x == "end").count(), 1, "seed {}", seed);
        } else {
            survived += 1;
            assert!(!k.contains(&"end"), "seed {}: end without death", seed);
        }
        assert!(!(r.outcome == "won" && !r.won), "seed {}: outcome won with won=false", seed);
        if r.outcome == "died" {
            assert!(!r.won, "seed {}", seed);
            assert_eq!(r.deaths, 1, "seed {}", seed);
        }
        if r.won && deaths == 1 {
            assert_eq!(r.outcome, "won", "seed {}: death after the boss keeps outcome won", seed);
            assert_eq!(r.post_deaths, 1, "seed {}", seed);
        }
    }
    assert!(died >= 20 && survived >= 20, "died {} survived {}: toggle must matter", died, survived);
}

#[test]
fn died_level_is_the_level_at_the_death_event() {
    let c = Config::default();
    let mut seen = 0;
    for seed in 0..100u64 {
        let r = simulate_with(seed, &c);
        if r.outcome == "died" {
            let d = r.events.iter().find(|e| e.kind == "death").expect("death event");
            assert_eq!(r.died_level, Some(d.hero.level), "seed {}", seed);
            seen += 1;
        } else {
            assert_eq!(r.died_level, None, "seed {}", seed);
        }
    }
    assert!(seen > 0);
}

#[test]
fn off_keeps_the_respawn_loop() {
    let c = cfg(&[("permadeath", "false"), ("monster_atk_mult", "3.0")]);
    let mut many = false;
    let mut respawns = 0;
    for seed in 0..40u64 {
        let r = simulate_with(seed, &c);
        respawns += r.events.iter().filter(|e| e.kind == "respawn").count();
        if r.deaths > 1 {
            many = true;
        }
        assert!(!r.events.iter().any(|e| e.kind == "end"), "seed {}: end event with permadeath off", seed);
    }
    assert!(respawns > 0, "no respawn events with permadeath off");
    assert!(many, "no run exceeded one death with permadeath off and monster_atk_mult 3.0");
}

#[test]
fn off_outcome_is_consistent() {
    let c = cfg(&[("permadeath", "false"), ("boss_attempts", "1"), ("monster_atk_mult", "1.5")]);
    let mut died = 0;
    for seed in 0..60u64 {
        let r = simulate_with(seed, &c);
        match r.outcome {
            "won" => assert!(r.won, "seed {}", seed),
            "capped" => assert!(r.capped && !r.won, "seed {}", seed),
            "died" => {
                assert!(!r.won && !r.capped, "seed {}", seed);
                died += 1;
            }
            o => panic!("seed {}: bad outcome {}", seed, o),
        }
        if !r.won && !r.capped {
            assert_eq!(r.outcome, "died", "seed {}: boss attempts exhausted must be died", seed);
        }
    }
    assert!(died > 0, "no exhausted-attempts run in 60 seeds");
}

#[test]
fn capped_alive_run_is_capped() {
    let c = cfg(&[("event_cap", "300"), ("monster_atk_mult", "0.1"), ("permadeath", "true")]);
    for seed in 1..=5u64 {
        let r = simulate_with(seed, &c);
        assert!(r.capped, "seed {}", seed);
        assert!(!r.won);
        assert_eq!(r.outcome, "capped", "seed {}", seed);
        assert_eq!(r.died_level, None);
        let j = parse(&run_body(&r)).unwrap();
        assert_eq!(j.get("outcome").unwrap().str(), "capped");
    }
}

#[test]
fn head_json_has_outcome_and_window_keeps_it() {
    let r = simulate_with(3, &Config::default());
    let j = parse(&run_body(&r)).unwrap();
    assert_eq!(j.get("outcome").unwrap().str(), r.outcome);
    let w = parse(&run_window_body(&r, 0, 10)).unwrap();
    assert_eq!(w.get("outcome").unwrap().str(), r.outcome);
}

#[test]
fn lean_summary_equals_full_summary() {
    for p in ["true", "false"] {
        let c = cfg(&[("permadeath", p), ("monster_atk_mult", "2.0")]);
        for seed in 0..30u64 {
            let r = simulate_with(seed, &c);
            let s = simulate_summary(seed, &c);
            assert_eq!(s.outcome, r.outcome, "seed {} permadeath {}", seed, p);
            assert_eq!(s.died_level, r.died_level, "seed {}", seed);
            assert_eq!(s.deaths, r.deaths, "seed {}", seed);
            assert_eq!(s.won, r.won, "seed {}", seed);
            assert_eq!(s.events as usize, r.events.len(), "seed {}", seed);
        }
    }
}

#[test]
fn cap_and_windowing_still_work() {
    let off = cfg(&[("permadeath", "false")]);
    let r = simulate_with(7, &off);
    assert_eq!(r.events.len(), 65536);
    assert!(r.capped);
    let w = parse(&run_window_body(&r, 0, 2000)).unwrap();
    assert_eq!(w.get("events").unwrap().arr().len(), 2000);
    // a permadeath run's last window ends with death then end
    let on = Config::default();
    let d = (0..200u64).map(|s| simulate_with(s, &on)).find(|r| r.outcome == "died").expect("a died run");
    let n = d.events.len();
    let w = parse(&run_window_body(&d, n - 2, 5)).unwrap();
    let ev = w.get("events").unwrap().arr();
    assert_eq!(ev.len(), 2);
    assert_eq!(ev[0].get("kind").unwrap().str(), "death");
    assert_eq!(ev[1].get("kind").unwrap().str(), "end");
}

#[test]
fn deterministic_both_modes() {
    for p in ["true", "false"] {
        let c = cfg(&[("permadeath", p), ("event_cap", "4000")]);
        for seed in [1u64, 9, 21] {
            assert_eq!(run_body(&simulate_with(seed, &c)), run_body(&simulate_with(seed, &c)), "seed {} {}", seed, p);
        }
    }
}

#[test]
fn batch_counts_add_up_and_stats_have_death_fields() {
    let s = batch_with(300, 4, &Config::default());
    assert_eq!(s.wins + s.died + s.capped, s.runs);
    assert!(s.died > 0);
    assert!((s.died_pct - s.died as f64 * 100.0 / s.runs as f64).abs() < 1e-6);
    assert_eq!(s.died_level_hist.iter().sum::<u32>(), s.died);
    assert_eq!(s.death_zones.iter().sum::<u32>(), s.died);
    let j = parse(&stats_body(&s)).unwrap();
    for k in ["died", "capped", "died_pct", "died_level_hist", "death_zones"] {
        assert!(j.get(k).is_some(), "stats missing {}", k);
    }
    assert_eq!(j.u("died") as u32, s.died);
    assert_eq!(j.get("died_level_hist").unwrap().arr().len(), s.died_level_hist.len());
}
