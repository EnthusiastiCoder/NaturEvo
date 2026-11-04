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

fn lerp_cap(a: &Capsule, b: &Capsule, t: f32) -> Capsule {
    let l = |p: [f32; 2], q: [f32; 2]| [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
    Capsule { a: l(a.a, b.a), b: l(a.b, b.b), r: a.r }
}

/// Deepest overlap (m) of the blades during this tick, 0 when they never touch (checked at 32 sub-steps).
fn depth(m: &Match) -> f32 {
    let (p0, c0) = (m.robots[0].sword_prev_capsule(), m.robots[0].sword_capsule());
    let (p1, c1) = (m.robots[1].sword_prev_capsule(), m.robots[1].sword_capsule());
    let mut worst = 0.0f32;
    for i in 1..=32 {
        let t = i as f32 / 32.0;
        let (a, b) = (lerp_cap(&p0, &c0, t), lerp_cap(&p1, &c1, t));
        let (_, _, d) = seg_seg(a.a, a.b, b.a, b.b);
        worst = worst.max(a.r + b.r - d);
    }
    worst
}
