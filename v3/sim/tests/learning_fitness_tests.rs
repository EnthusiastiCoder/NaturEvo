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
