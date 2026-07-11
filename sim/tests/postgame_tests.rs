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
