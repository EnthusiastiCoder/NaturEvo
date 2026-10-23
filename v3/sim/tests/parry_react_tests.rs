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

fn count(j: &str, kind: &str) -> usize {
    j.matches(&format!("\"kind\":\"{}\"", kind)).count()
}

#[test]
fn skillbot_parries_open_riposte_windows() {
    let mut n = 0;
    for tier in [(2u8, 2u8), (3, 2), (2, 3), (3, 3)] {
        for s in 0..20u64 {
            n += count(&run(Brain::Skill(tier.0), Brain::Skill(tier.1), s), "riposte_open");
        }
    }
    assert!(n >= 8, "riposte_open fired only {} times in 80 skillbot fights", n);
}
