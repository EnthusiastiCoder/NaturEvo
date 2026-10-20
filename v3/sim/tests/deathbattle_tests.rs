// Job 3: fights are death battles. They end only when someone dies; nothing forces engagement in-fight; only the capped-loss fitness pushes it.
use mech_sim::config::*;
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::fitness::{score_stats, stall_units, MatchStats};
use mech_sim::genome::Genome;
use mech_sim::log::fight_json;
use mech_sim::rng::Rng;
use mech_sim::robot::{Part, Role};

fn dead(m: &Match) -> bool {
    m.robots.iter().any(|r| r.is_ko())
}

#[test]
fn dummies_that_never_fight_hit_the_cap_not_a_forced_death() {
    // No walls, no bleed: nothing forces a death, so two passive fighters run to the emergency cap.
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let r = m.run_to_end();
    assert!(!dead(&m), "nothing may kill passive fighters (ended at {})", r.ticks);
    assert_eq!(r.ticks, HARD_CAP_TICKS);
    assert!(r.capped && r.outcome == Outcome::Draw);
}

#[test]
fn no_fight_ends_without_a_death_or_the_cap_across_many_matchups() {
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
        assert!(dead(&m) != r.capped, "seed {} ended at {}: a fight ends by a death XOR the cap", seed, r.ticks);
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

#[test]
fn arena_width_is_constant_through_a_fight() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let (lo0, hi0) = m.bounds();
    assert!((hi0 - lo0 - (ARENA_WIDTH - 1.0)).abs() < 1e-3);
    while m.tick < SUDDEN_DEATH_START {
        m.step();
    }
    let (lo, hi) = m.bounds();
    assert!((hi - lo - (hi0 - lo0)).abs() < 1e-3, "no closing during the grace period");
    for _ in 0..120 {
        m.step();
    }
    let (lo2, hi2) = m.bounds();
    assert!(hi2 - lo2 < hi - lo - 0.5 && lo2 > lo && hi2 < hi);
    for i in 0..2 {
        assert!(m.robots[i].x >= lo2 - 1e-3 && m.robots[i].x <= hi2 + 1e-3, "fighter {} outside the walls", i);
    }
    while !m.is_over() && m.tick < SUDDEN_DEATH_START + 1500 {
        m.step();
    }
    let (lo3, hi3) = m.bounds();
    assert!(hi3 - lo3 >= WALL_MIN_WIDTH - 1e-3, "walls stop at the minimum width");
}

#[test]
fn sudden_death_bleed_escalates_and_starts_only_after_the_grace_period() {
    let mut early = Match::new(1, Brain::Dummy, Brain::Dummy);
    for _ in 0..SUDDEN_DEATH_START - 5 {
        early.step();
    }
    assert_eq!(early.robots[0].hp(Part::Torso), early.robots[0].max_hp(Part::Torso), "no bleed before sudden death");
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    while m.tick < SUDDEN_DEATH_START + 60 {
        m.step();
    }
    let torso = |m: &Match| m.robots[0].hp(Part::Torso);
    let a = torso(&m);
    for _ in 0..120 {
        m.step();
    }
    let b = torso(&m);
    for _ in 0..120 {
        m.step();
    }
    let c = torso(&m);
    assert!(a - b > 0.0 && (b - c) > (a - b), "bleed must grow: {} {} {}", a, b, c);
}

#[test]
fn a_death_win_beats_every_non_death_result() {
    let win = MatchStats { win: true, ko: true, ..Default::default() };
    let draw = MatchStats { draw: true, ..Default::default() };
    assert!(score_stats(&win) > score_stats(&draw));
    let loss = MatchStats { dmg_taken_norm: 1.0, ..Default::default() };
    assert!(score_stats(&loss) > score_stats(&draw), "a loss still beats a draw");
}

#[test]
fn log_contains_the_whole_fight_to_the_death() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    m.enable_log();
    let r = m.run_to_end();
    let j = fight_json(&m);
    assert_eq!(j.matches("\"f\":[").count() as u32, r.ticks / 2 + 1, "one frame per 2 ticks plus the final state");
    assert!(j.contains("\"kind\":\"ko\""), "the killing blow is logged");
    assert!(j.contains("\"kind\":\"sudden_death\""));
    assert!(j.contains("\"sudden_death\":{\"start_tick\":"));
    assert!(j.contains("\"state\":\"ko\""), "the last frame shows the dead fighter");
}
