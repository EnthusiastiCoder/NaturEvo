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
