// r7: style identity in training fitness. Move share entropy (not distinct count), footwork, spacing spread,
// counter punish, and graded capped-fight loss. Nothing here pays for guarding, blocking or parrying.
use mech_sim::duel::{Brain, Match, MatchResult, Outcome};
use mech_sim::evolve::style_bonus;
use mech_sim::fitness::move_entropy;
use mech_sim::moveset::MoveId;
use mech_sim::stylebot::StyleId;
