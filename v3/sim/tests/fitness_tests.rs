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
