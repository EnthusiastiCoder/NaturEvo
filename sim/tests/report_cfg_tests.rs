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
