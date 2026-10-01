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
