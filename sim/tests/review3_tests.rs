//! Third browser review: consistent end event with permadeath off, cumulative counters in run windows.
mod common;
use common::parse;
use jungle_sim::config::Config;
use jungle_sim::report::run_window_body;
use jungle_sim::sim::simulate_with;

#[test]
fn permadeath_off_died_run_ends_with_an_end_event() {
    let mut c = Config::default();
    c.set("permadeath", "false").unwrap();
    let mut died = 0;
    for seed in 1..=40u64 {
        let r = simulate_with(seed, &c);
        if r.outcome == "died" {
            died += 1;
            assert_eq!(r.events.last().unwrap().kind, "end", "seed {seed}: died run must end with an end event");
            assert_eq!(r.events[r.events.len() - 2].kind, "respawn", "seed {seed}: death and respawn precede the end");
        } else {
            assert_ne!(r.events.last().unwrap().kind, "end", "seed {seed}");
        }
    }
    assert!(died > 0, "expected at least one died run in 40 seeds with permadeath off");
}

#[test]
fn window_carries_prefix_counters() {
    let mut c = Config::default();
    c.set("permadeath", "false").unwrap();
    let r = simulate_with(1, &c);
    let from = r.events.len() / 2;
    let v = parse(&run_window_body(&r, from, 100)).unwrap();
    let n = |k: &str| r.events[..from].iter().filter(|e| e.kind == k).count() as u64;
    assert_eq!(v.get("prefix").unwrap().get("fights").unwrap().num() as u64, n("victory_fight"));
    assert_eq!(v.get("prefix").unwrap().get("deaths").unwrap().num() as u64, n("death"));
    assert_eq!(v.get("prefix").unwrap().get("skills").unwrap().num() as u64, n("skill"));
    assert!(n("victory_fight") > 0);
}
