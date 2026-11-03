//! Bodies are solid: two fighters can neither overlap nor swap sides, and a staggered fighter recovers control quickly.
use mech_sim::arena::Arena;
use mech_sim::config::{BODY_MIN_SEP, STAGGER_LOCK, STAGGER_TIME};
use mech_sim::duel::{Brain, Match};
use mech_sim::robot::{Role, Robot, State, DT, STAND_TARGETS};
use mech_sim::stylebot::StyleId;

fn fights() -> Vec<(Brain, Brain)> {
    vec![
        (Brain::Scripted, Brain::Skill(2)),
        (Brain::Skill(1), Brain::Skill(3)),
        (Brain::Style(StyleId::Press), Brain::Style(StyleId::Counter)),
        (Brain::Style(StyleId::Duelist), Brain::Style(StyleId::Press)),
        (Brain::Skill(3), Brain::Skill(3)),
    ]
}
