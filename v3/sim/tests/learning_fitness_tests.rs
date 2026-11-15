// Learning-side anti-stalemate fitness: pressure when capped, finishing speed, accuracy, punish and landed variety.
// None of these pays for guarding, blocking or parrying; none is an in-fight rule.
use mech_sim::curriculum::{opponent_mix, OppKind};
use mech_sim::duel::{Brain, Match};
use mech_sim::fitness::*;

fn capped(pressure: f32) -> MatchStats {
    MatchStats { capped: true, pressure, ..Default::default() }
}

#[test]
fn capped_fight_pays_the_side_that_pressed_more() {
    assert!(score_stats(&capped(1.0)) > score_stats(&capped(0.0)) + 1.0);
}

#[test]
fn capped_stays_below_the_worst_death_loss_even_with_full_pressure() {
    let worst_loss = MatchStats { dmg_taken_norm: 1.0, idle_guard: 1.0, passivity: 1.0, stall: 100.0, ..Default::default() };
    assert!(score_stats(&capped(1.0)) < score_stats(&worst_loss), "{} vs {}", score_stats(&capped(1.0)), score_stats(&worst_loss));
}

#[test]
fn faster_kill_scores_higher() {
    let slow = MatchStats { win: true, ko: true, ticks_frac: 0.9, ..Default::default() };
    let fast = MatchStats { win: true, ko: true, ticks_frac: 0.1, ..Default::default() };
    assert!(score_stats(&fast) > score_stats(&slow) + 0.5);
}

#[test]
fn accuracy_punish_and_landed_variety_are_paid() {
    let base = MatchStats::default();
    assert!(score_stats(&MatchStats { accuracy: 0.5, ..base }) > score_stats(&base));
    assert!(score_stats(&MatchStats { punish: 0.5, ..base }) > score_stats(&base));
    let one = score_stats(&MatchStats { landed_moves: 1, ..base });
    let three = score_stats(&MatchStats { landed_moves: 3, ..base });
    let nine = score_stats(&MatchStats { landed_moves: 9, ..base });
    let four = score_stats(&MatchStats { landed_moves: 4, ..base });
    assert!(three > one && four > three);
    assert_eq!(nine, four, "variety credit saturates at 4 moves");
}

#[test]
fn defence_weight_is_unchanged() {
    assert_eq!(W_DEFENCE, 2.0);
}

#[test]
fn match_result_reports_swings_punish_and_landed_moves() {
    let r = Match::new(4, Brain::Skill(2), Brain::Dummy).run_to_end();
    assert!(r.swings[0] > 0);
    assert!(r.landed_moves[0] >= 1, "hits {:?} swings {:?} skill bot lands at least one move type on a dummy", r.hits, r.swings);
    assert_eq!(r.swings[1], 0);
    let s = stats_for(&r, 0);
    assert!(s.pressure > 0.0 && s.accuracy > 0.0);
    assert!(s.ticks_frac > 0.0 && s.ticks_frac <= 1.0);
}

#[test]
fn stage_c_mix_has_more_aggressive_bots_and_less_self_play() {
    let mix = opponent_mix(500);
    let w = |f: &dyn Fn(OppKind) -> bool| mix.iter().filter(|(k, _)| f(*k)).map(|(_, w)| *w).sum::<f32>();
    let total: f32 = mix.iter().map(|m| m.1).sum();
    assert!((total - 1.0).abs() < 1e-5);
    assert!(w(&|k| k == OppKind::SelfPlay) <= 0.35);
    assert!(w(&|k| matches!(k, OppKind::Skill(_))) >= 0.4);
    assert!(mix.iter().any(|(k, _)| *k == OppKind::Skill(2)));
    assert!(w(&|k| k == OppKind::Hof) >= 0.2);
}

#[test]
fn stage_c_keeps_a_passive_target_so_finishing_is_trained() {
    let mix = opponent_mix(500);
    assert!(mix.iter().any(|(k, w)| *k == OppKind::Dummy && *w >= 0.05));
}
