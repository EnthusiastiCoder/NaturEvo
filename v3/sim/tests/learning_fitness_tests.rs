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
