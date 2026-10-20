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
    while m.tick < 2000 {
        m.step();
        assert_eq!(m.bounds(), (lo0, hi0), "walls moved at tick {}", m.tick);
    }
}

#[test]
fn nobody_bleeds_without_being_hit() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    while m.tick < 1500 {
        m.step();
    }
    for i in 0..2 {
        for p in Part::ALL {
            assert_eq!(m.robots[i].hp(p), m.robots[i].max_hp(p), "fighter {} {:?} lost hp with no hit", i, p);
        }
    }
}

#[test]
fn stamina_regen_never_changes_over_time() {
    // No time-based rule: a rested idle fighter regens at the same rate late as early.
    let regen = |skip: u32| {
        let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
        while m.tick < skip {
            m.step();
        }
        m.robots[0].energy = 10.0;
        for _ in 0..60 {
            m.step();
        }
        m.robots[0].energy - 10.0
    };
    let (early, late) = (regen(10), regen(3000));
    assert!(early > 3.0 && (early - late).abs() < 1e-3, "early {} late {}", early, late);
}

#[test]
fn a_capped_fight_scores_below_a_lost_fight() {
    let capped = MatchStats { capped: true, draw: true, ..Default::default() };
    let worst_loss = MatchStats { dmg_taken_norm: 1.0, idle_guard: 1.0, passivity: 1.0, jerk: 0.3, ..Default::default() };
    assert!(score_stats(&capped) < score_stats(&worst_loss));
    assert!(score_stats(&capped) < DRAW_SCORE, "below a mutual death too");
    let busy = MatchStats { capped: true, draw: true, dmg_dealt_norm: 0.0, dmg_taken_norm: 1.0, ..Default::default() };
    assert!(score_stats(&busy) < score_stats(&worst_loss), "even the worst capped fight loses to the worst death-loss");
    let mut m = Match::new(2, Brain::Dummy, Brain::Dummy);
    let r = m.force_cap_for_test();
    assert!(mech_sim::fitness::score_match(&r, 0) < score_stats(&worst_loss));
    assert!(mech_sim::fitness::score_match(&r, 1) < score_stats(&worst_loss));
    assert_eq!(mech_sim::fitness::score_match(&r, 0), mech_sim::fitness::score_match(&r, 1), "mutual");
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
    let mut m = Match::new(6, Brain::Scripted, Brain::Dummy);
    m.enable_log();
    let r = m.run_to_end();
    let j = fight_json(&m);
    assert_eq!(j.matches("\"f\":[").count() as u32, r.ticks / 2 + 1, "one frame per 2 ticks plus the final state");
    assert!(j.contains("\"kind\":\"ko\""), "the killing blow is logged");
    assert!(!j.contains("sudden_death"), "no sudden-death block or event");
    assert!(j.contains("\"geometry\":"), "the geometry block stays");
    assert!(j.contains("\"capped\":false"));
    assert!(j.contains("\"state\":\"ko\""), "the last frame shows the dead fighter");
}

#[test]
fn log_marks_a_capped_fight() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    m.enable_log();
    let r = m.run_to_end();
    assert!(r.capped);
    assert!(fight_json(&m).contains("\"capped\":true"));
}

#[test]
fn stall_penalty_is_monotone_in_stall_time_and_zero_for_frequent_hits() {
    assert_eq!(stall_units(0), 0.0);
    assert_eq!(stall_units(STALL_GRACE_TICKS), 0.0, "a short no-hit stretch is free");
    let mut prev = 0.0;
    for len in (STALL_GRACE_TICKS..6000).step_by(300) {
        let u = stall_units(len);
        assert!(u >= prev, "monotone at {}", len);
        prev = u;
    }
    assert!(stall_units(2000) > 2.0 * stall_units(1000) - 1e-6, "superlinear: long stalls cost more than the same time in pieces");
    let some = MatchStats { stall: stall_units(1500), ..Default::default() };
    let none = MatchStats::default();
    assert!(score_stats(&some) < score_stats(&none));
    let more = MatchStats { stall: stall_units(3000), ..Default::default() };
    assert!(score_stats(&more) < score_stats(&some));
    let huge = MatchStats { stall: 1e6, ..Default::default() };
    assert!(score_stats(&huge) >= score_stats(&none) - W_STALL_MAX - 1e-4, "bounded");
    let worst_loss = MatchStats { dmg_taken_norm: 1.0, idle_guard: 1.0, passivity: 1.0, jerk: 0.3, stall: 1e6, ..Default::default() };
    let capped = MatchStats { capped: true, draw: true, dmg_dealt_norm: 0.0, dmg_taken_norm: 0.0, ..Default::default() };
    assert!(score_stats(&capped) < score_stats(&worst_loss), "capped stays the worst case");
}

#[test]
fn stall_is_measured_per_fight_for_both_and_changes_nothing_in_the_fight() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let r = m.run_to_end();
    assert!(r.stall > 0.0 && (r.stall - stall_units(HARD_CAP_TICKS)).abs() < 1e-3, "{}", r.stall);
    assert_eq!(mech_sim::fitness::stats_for(&r, 0).stall, mech_sim::fitness::stats_for(&r, 1).stall);
    // Busy fight: hits keep landing, the stall is far smaller than a whole-fight stall.
    let busy = Match::new(6, Brain::Scripted, Brain::Dummy).run_to_end();
    assert!(busy.stall < stall_units(busy.ticks));
    // Reading the fitness changes nothing: identical fights score-independently.
    let a = Match::new(6, Brain::Scripted, Brain::Dummy).run_to_end();
    assert_eq!((a.ticks, a.hits), (busy.ticks, busy.hits));
}
