//! Posture: the ready stance is a natural upright fighting stance and fights do not hold a heavy lean.
use mech_sim::duel::{Brain, Match};
use mech_sim::robot::{Joint, Part, Robot, Role, State, DT, STAND_TARGETS};

/// Forward torso tilt in degrees from the capsule geometry (what the viewer draws); positive = toward facing.
fn torso_tilt(r: &Robot) -> f32 {
    let c = r.capsule(Part::Torso);
    ((c.b[0] - c.a[0]) * r.facing).atan2(c.b[1] - c.a[1]).to_degrees()
}

#[test]
fn ready_pose_is_upright_with_slightly_bent_knees() {
    let mut r = Robot::new(Role::Hero);
    for _ in 0..200 {
        r.apply_targets(&STAND_TARGETS, DT);
    }
    let t = torso_tilt(&r);
    assert!((0.0..=8.0).contains(&t), "torso tilt {t} deg: want a small forward lean (0..8)");
    let k = r.joint_angle(Joint::KneeL).to_degrees();
    assert!((6.0..=25.0).contains(&k), "knee {k} deg: want slightly bent (6..25)");
    let head = r.capsule(Part::Head);
    let torso = r.capsule(Part::Torso);
    assert!(head.a[1] > torso.b[1], "head above the torso top");
    assert!((head.a[0] - torso.b[0]).abs() * r.facing.signum() < 0.25, "head not thrust forward");
    // Neck keeps the head over the shoulders.
    assert!(r.joint_angle(Joint::Neck).to_degrees().abs() < 8.0);
}

#[test]
fn standing_robot_stays_upright_and_balanced() {
    let mut r = Robot::new(Role::Hero);
    for i in 0..(5.0 / DT) as usize {
        r.apply_targets(&STAND_TARGETS, DT);
        if i > 100 {
            assert_eq!(r.state(), State::Idle);
            assert!(torso_tilt(&r) < 8.0 && torso_tilt(&r) > -2.0);
            assert!(r.balance_margin() > 0.1, "CoM over the feet, margin {}", r.balance_margin());
        }
    }
}
