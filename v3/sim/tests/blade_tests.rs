//! Blades are solid against each other: no tick ends with the two blades overlapping, and no blade passes through the
//! other between ticks (swept test), whatever the fighters do. The same for both fighters.
use mech_sim::arena::Arena;
use mech_sim::combat::seg_seg;
use mech_sim::duel::{Brain, Match};
use mech_sim::moveset::MoveId;
use mech_sim::robot::Capsule;
use mech_sim::stylebot::StyleId;

fn fights() -> Vec<(Brain, Brain)> {
    vec![
        (Brain::Scripted, Brain::Skill(2)),
        (Brain::Skill(2), Brain::Skill(3)),
        (Brain::Style(StyleId::Press), Brain::Style(StyleId::Counter)),
        (Brain::Style(StyleId::Duelist), Brain::Style(StyleId::Press)),
        (Brain::Style(StyleId::Counter), Brain::Style(StyleId::Counter)),
        (Brain::Skill(3), Brain::Skill(3)),
        (Brain::Spam(MoveId::Overhead), Brain::Dummy),
        (Brain::Cycle(vec![MoveId::DiagUp, MoveId::Thrust, MoveId::Sweep]), Brain::Spam(MoveId::Parry)),
    ]
}
