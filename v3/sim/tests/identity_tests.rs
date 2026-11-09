// r7: style identity in training fitness. Move share entropy (not distinct count), footwork, spacing spread,
// counter punish, and graded capped-fight loss. Nothing here pays for guarding, blocking or parrying.
use mech_sim::duel::{Brain, Match, MatchResult, Outcome};
use mech_sim::evolve::style_bonus;
use mech_sim::fitness::move_entropy;
use mech_sim::moveset::MoveId;
use mech_sim::stylebot::StyleId;

fn base() -> MatchResult {
    let mut m = Match::new(3, Brain::Skill(2), Brain::Skill(1));
    let mut r = m.run_to_end();
    r.capped = false;
    r.outcome = Outcome::WinA;
    r.landed_moves = [3, 3];
    r.swings = [12, 12];
    r.move_counts = [[0; 14]; 2];
    r.hit_counts = [[0; 14]; 2];
    for k in [1usize, 2, 3, 4, 5, 6] {
        r.move_counts[0][k] = 2;
        r.hit_counts[0][k] = 1;
    }
    r.still_share = [0.1, 0.1];
    r.fwd_share = [0.3, 0.3];
    r.engaged_std = 0.6;
    r.counter_hits = [2, 0];
    r.backstep_hits = [1, 0];
    r
}
