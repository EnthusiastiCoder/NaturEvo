use jungle_sim::report::{batch, run_json, stats_json};
use jungle_sim::sim::{simulate, Event, HeroSnap, Run};

#[test]
fn run_json_shape_and_deterministic() {
    let a = run_json(&simulate(5));
    let b = run_json(&simulate(5));
    assert!(a.starts_with("window.RUN"));
    assert!(a.trim_end().ends_with(';'));
    assert_eq!(a.as_bytes(), b.as_bytes());
}
