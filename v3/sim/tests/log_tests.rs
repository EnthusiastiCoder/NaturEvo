// W3 log format (plan section 6). The log is hand-rolled JSON, tests inspect it textually (no third-party crates).
use mech_sim::duel::{Brain, Match};
use mech_sim::log::fight_json;

fn logged(a: Brain, b: Brain, seed: u64) -> (String, u32) {
    let mut m = Match::new(seed, a, b);
    m.enable_log();
    let r = m.run_to_end();
    (fight_json(&m), r.ticks)
}
