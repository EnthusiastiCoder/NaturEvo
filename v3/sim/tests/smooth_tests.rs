// Job 2: smooth motion. Motors have a bounded acceleration and approach targets without overshoot, the net's
// action is low-passed, fitness carries a small jerk cost, and the smoothness metric itself is sane.
mod common;
use common::*;
use mech_sim::config::*;
use mech_sim::diag::SmoothAcc;
use mech_sim::duel::{smooth_toward, Brain, Match};
use mech_sim::fitness::{score_stats, MatchStats, W_JERK};
use mech_sim::robot::{Joint, Role, DT, STAND_TARGETS};

#[test]
fn motors_limit_acceleration_and_do_not_overshoot() {
    let mut r = standing(Role::Hero, 4.0, 1.0);
    let mut t = STAND_TARGETS;
    t[Joint::ShoulderR as usize] = 1.0;
    let target = Joint::ShoulderR.limits().1;
    let mut prev = r.joint_vel;
    let mut prev_ang = r.joint_angle(Joint::ShoulderR);
    for _ in 0..120 {
        r.apply_targets(&t, DT);
        for i in 0..12 {
            assert!((r.joint_vel[i] - prev[i]).abs() <= MOTOR_ACCEL * DT * 1.001 + 1e-4, "joint {} dv {}", i, r.joint_vel[i] - prev[i]);
        }
        prev = r.joint_vel;
        let a = r.joint_angle(Joint::ShoulderR);
        assert!(a >= prev_ang - 1e-5 && a <= target + 1e-4, "monotone approach, no overshoot: {} -> {}", prev_ang, a);
        prev_ang = a;
    }
    assert!((prev_ang - target).abs() < 0.02, "reaches the target: {} vs {}", prev_ang, target);
    t[Joint::ShoulderR as usize] = -1.0;
    for _ in 0..30 {
        r.apply_targets(&t, DT);
        for i in 0..12 {
            assert!((r.joint_vel[i] - prev[i]).abs() <= MOTOR_ACCEL * DT * 1.001 + 1e-4);
        }
        prev = r.joint_vel;
    }
}

#[test]
fn smooth_toward_is_a_first_order_low_pass() {
    let a = [0.0f32; 12];
    let mut b = [0.0f32; 12];
    b[3] = 1.0;
    let c = smooth_toward(&a, &b, 0.25);
    assert!((c[3] - 0.25).abs() < 1e-6 && c[0] == 0.0);
    let mut x = a;
    for _ in 0..60 {
        x = smooth_toward(&x, &b, ACTION_SMOOTH);
    }
    assert!((x[3] - 1.0).abs() < 0.01);
    assert!(ACTION_SMOOTH > 0.0 && ACTION_SMOOTH < 1.0);
}
