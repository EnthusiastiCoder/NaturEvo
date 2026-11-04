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

#[test]
fn overlapping_bodies_are_pushed_apart_equally() {
    let mut m = Match::new(3, Brain::Dummy, Brain::Dummy);
    m.arena = Arena::flat(m.arena.width);
    m.robots[0].x = 6.0;
    m.robots[1].x = 6.2;
    m.robots[0].fk();
    m.robots[1].fk();
    m.step();
    let d = m.robots[1].x - m.robots[0].x;
    assert!(d >= BODY_MIN_SEP - 1e-3, "separation {d}");
    let mid = (m.robots[0].x + m.robots[1].x) / 2.0;
    assert!((mid - 6.1).abs() < 0.05, "push must be symmetric, midpoint {mid}");
}

#[test]
fn walking_into_each_other_stops_at_body_contact() {
    // Two scripted walkers heading for each other cannot pass through.
    let mut m = Match::new(9, Brain::Scripted, Brain::Scripted);
    m.arena = Arena::flat(m.arena.width);
    m.robots[0].x = 6.0;
    m.robots[1].x = 6.3;
    m.robots[0].fk();
    m.robots[1].fk();
    for _ in 0..300 {
        m.step();
        assert!(m.robots[1].x - m.robots[0].x >= BODY_MIN_SEP - 1e-3, "tick {} d {}", m.tick, m.robots[1].x - m.robots[0].x);
    }
}

fn staggered() -> Robot {
    let mut r = Robot::new(Role::Hero);
    r.x = 3.0;
    r.facing = 1.0;
    for _ in 0..10 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    r.add_posture(150.0);
    r
}

#[test]
fn stagger_lock_is_short_for_both_roles() {
    assert!(STAGGER_LOCK <= 0.35 && STAGGER_LOCK < STAGGER_TIME);
    for role in [Role::Hero, Role::Monster] {
        let mut r = staggered();
        r = if role == Role::Hero { r } else { let mut m = Robot::new(Role::Monster); m.x = 3.0; m.facing = 1.0; for _ in 0..10 { m.apply_targets(&STAND_TARGETS, DT); } m.add_posture(150.0); m };
        assert_eq!(r.state(), State::Staggered);
        assert!(!r.can_act(), "locked right after the blow");
        let n = (STAGGER_LOCK / DT) as usize + 2;
        for _ in 0..n {
            r.apply_targets(&STAND_TARGETS, DT);
        }
        assert!(r.can_act(), "{role:?} must act again within {STAGGER_LOCK} s");
        assert_eq!(r.state(), State::Staggered, "still exposed (x1.5 damage) after the lock");
    }
}
