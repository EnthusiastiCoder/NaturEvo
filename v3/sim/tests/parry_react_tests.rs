// A Parry started in reaction to a visible cut meets the cut and opens the riposte window; no Parry, no window.
use mech_sim::duel::{Brain, Match};
use mech_sim::log::fight_json;
use mech_sim::moveset::REST;

fn run(a: Brain, b: Brain, seed: u64) -> String {
    let mut m = Match::new(seed, a, b);
    m.enable_log();
    m.run_to_end();
    fight_json(&m)
}
