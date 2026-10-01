// W4 fitness: draw worse than loss, defence credit rules, idle guard, zone weights.
use mech_sim::config::{DRAW_SCORE, IMPACT_REF};
use mech_sim::duel::{Brain, Match, Resolution};
use mech_sim::fitness::{defence_credit, score_match, score_stats, zone_weight, MatchStats, ThreatOutcome};
use mech_sim::robot::{Part, GUARD_TARGETS};

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}
fn th(r: Resolution, i: f32, followup: bool) -> ThreatOutcome {
    ThreatOutcome { resolution: r, i_potential: i, followup_hit: followup }
}

#[test]
fn score_formula() {
    let s = MatchStats { win: true, dmg_dealt_norm: 0.5, dmg_taken_norm: 0.2, defence_credit: 0.5, idle_guard: 0.1, passivity: 0.2, chain_quality: 0.4, ..Default::default() };
    // 4 + 3*0.5 - 2.5*0.2 + 2*0.5 - 1*0.1 - 1*0.2 + 0.5*0.4
    approx(score_stats(&s), 4.0 + 1.5 - 0.5 + 1.0 - 0.1 - 0.2 + 0.2, 1e-4);
}

#[test]
fn draw_is_minus_6_and_strictly_worse_than_any_loss() {
    assert_eq!(DRAW_SCORE, -6.0);
    let draw = MatchStats { draw: true, dmg_dealt_norm: 1.0, ..Default::default() };
    assert_eq!(score_stats(&draw), -6.0, "draw ignores every other term");
    let worst_loss = MatchStats { dmg_taken_norm: 1.0, idle_guard: 1.0, passivity: 1.0, ..Default::default() };
    assert!(score_stats(&worst_loss) > score_stats(&draw), "{} vs {}", score_stats(&worst_loss), score_stats(&draw));
}

#[test]
fn ko_bonus_is_paid_only_on_ko() {
    let base = MatchStats { win: true, ..Default::default() };
    let ko = MatchStats { win: true, ko: true, ..Default::default() };
    assert!(score_stats(&ko) > score_stats(&base));
}

#[test]
fn defence_credit_values_per_resolution() {
    approx(defence_credit(&[th(Resolution::Parried, IMPACT_REF, false)]), 1.0, 1e-4);
    approx(defence_credit(&[th(Resolution::Parried, IMPACT_REF * 0.5, false)]), 0.5, 1e-4);
    approx(defence_credit(&[th(Resolution::Parried, IMPACT_REF * 5.0, false)]), 1.0, 1e-4);
    approx(defence_credit(&[th(Resolution::Blocked, IMPACT_REF, false)]), 0.7, 1e-4);
    approx(defence_credit(&[th(Resolution::Dodged, IMPACT_REF, false)]), 0.7, 1e-4);
    assert_eq!(defence_credit(&[th(Resolution::Hit, IMPACT_REF, false)]), 0.0);
    assert_eq!(defence_credit(&[th(Resolution::Miss, IMPACT_REF, false)]), 0.0);
}

#[test]
fn credit_is_paid_once_per_threat_and_capped_at_6_threats() {
    let one = defence_credit(&[th(Resolution::Parried, IMPACT_REF, false)]);
    let three = defence_credit(&vec![th(Resolution::Parried, IMPACT_REF, false); 3]);
    approx(three, 3.0 * one, 1e-4);
    let ten = defence_credit(&vec![th(Resolution::Parried, IMPACT_REF, false); 10]);
    approx(ten, 6.0, 1e-4);
}

#[test]
fn credit_is_halved_when_a_follow_up_hit_lands_within_half_a_second() {
    approx(defence_credit(&[th(Resolution::Parried, IMPACT_REF, true)]), 0.5, 1e-4);
    approx(defence_credit(&[th(Resolution::Blocked, IMPACT_REF, true)]), 0.35, 1e-4);
}

#[test]
fn zone_weights_head_over_core_over_limbs() {
    approx(zone_weight(Part::Head), 1.5, 1e-5);
    approx(zone_weight(Part::Torso), 1.0, 1e-5);
    approx(zone_weight(Part::Pelvis), 1.0, 1e-5);
    for p in [Part::UArmL, Part::FArmR, Part::ThighL, Part::ShinR, Part::FootL] {
        approx(zone_weight(p), 0.7, 1e-5);
    }
}

#[test]
fn pure_turtle_scores_below_a_fighter_that_engages() {
    let turtle = Match::new(4, Brain::Fixed(GUARD_TARGETS), Brain::Dummy).run_to_end();
    let engager = Match::new(4, Brain::Scripted, Brain::Dummy).run_to_end();
    assert!(turtle.idle_guard_share[0] > 0.9, "turtle holds guard all match: {}", turtle.idle_guard_share[0]);
    assert!(score_match(&turtle, 0) < score_match(&engager, 0));
}
