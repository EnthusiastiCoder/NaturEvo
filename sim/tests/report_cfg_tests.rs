use jungle_sim::config::Config;
use jungle_sim::report::{batch, batch_with, run_body, run_json, stats_body, stats_json};
use jungle_sim::sim::simulate;

#[test]
fn body_wrappers_equal_json() {
    let r = simulate(3);
    assert_eq!(run_json(&r), format!("window.RUN = {};\n", run_body(&r)));
    let s = batch(20, 2);
    assert_eq!(stats_json(&s), format!("window.STATS = {};\n", stats_body(&s)));
}

#[test]
fn batch_with_default_equals_batch() {
    let a = batch(60, 2);
    let b = batch_with(60, 2, &Config::default());
    assert_eq!(a.runs, b.runs);
    assert_eq!(a.wins, b.wins);
    assert_eq!(a.avg_deaths, b.avg_deaths);
    assert_eq!(a.avg_final_level, b.avg_final_level);
    assert_eq!(a.level_hist, b.level_hist);
}
