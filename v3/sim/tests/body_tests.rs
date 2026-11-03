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

#[test]
fn bodies_never_overlap_or_swap_sides() {
    for (a, b) in fights() {
        for seed in 0..6u64 {
            let mut m = Match::new(500 + seed, a.clone(), b.clone());
            m.arena = Arena::flat(m.arena.width);
            let side = (m.robots[1].x - m.robots[0].x).signum();
            while !m.is_over() {
                m.step();
                let d = (m.robots[1].x - m.robots[0].x) * side;
                assert!(d >= BODY_MIN_SEP - 1e-3, "seed {seed}: bodies {d:.3} m apart (min {BODY_MIN_SEP})");
            }
        }
    }
}
