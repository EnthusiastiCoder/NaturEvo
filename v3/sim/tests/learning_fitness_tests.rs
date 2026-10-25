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
