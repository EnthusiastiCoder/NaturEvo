// Job 3: fights are death battles. They end only when someone dies; sudden death forces engagement.
use mech_sim::config::*;
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::fitness::{score_stats, MatchStats};
use mech_sim::genome::Genome;
use mech_sim::log::fight_json;
use mech_sim::rng::Rng;
use mech_sim::robot::{Part, Role};

fn dead(m: &Match) -> bool {
    m.robots.iter().any(|r| r.is_ko())
}

#[test]
fn dummies_never_time_out_they_die_in_sudden_death() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let r = m.run_to_end();
    assert!(dead(&m), "ended at {} without a death", r.ticks);
    assert!(r.ticks > SUDDEN_DEATH_START, "nothing forces a death before sudden death: {}", r.ticks);
    assert!(r.ticks < HARD_CAP_TICKS);
    assert!(!r.capped);
    assert!(r.sudden_death);
}
