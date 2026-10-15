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

#[test]
fn no_fight_ends_without_a_death_across_many_matchups() {
    for seed in 0..60u64 {
        let mut rng = Rng::new(seed * 11 + 3);
        let a = Genome::random(Role::Hero, &mut rng);
        let b = Genome::random(Role::Monster, &mut rng);
        let (ba, bb) = match seed % 4 {
            0 => (Brain::Net(a), Brain::Net(b)),
            1 => (Brain::Net(a), Brain::Dummy),
            2 => (Brain::Scripted, Brain::Net(b)),
            _ => (Brain::Scripted, Brain::Scripted),
        };
        let mut m = Match::new(seed, ba, bb);
        let r = m.run_to_end();
        assert!(dead(&m) && !r.capped, "seed {} ended at {} without a death", seed, r.ticks);
    }
}

#[test]
fn a_timeout_is_never_a_win() {
    // The emergency cap yields a draw, never a win by hp lead.
    let mut m = Match::new(2, Brain::Dummy, Brain::Dummy);
    m.robots[0].set_hp(Part::Head, 1.0);
    let r = m.force_cap_for_test();
    assert!(r.capped && r.outcome == Outcome::Draw);
}
