// W3 sensing: input vector length, finiteness, mirroring, head damage.
mod common;
use common::{arena, standing};
use mech_sim::robot::{Joint, Part, Robot, Role, DT, STAND_TARGETS};
use mech_sim::rng::Rng;
use mech_sim::sense::{observe, N_IN, OPP_HP, OPP_POINTS};

#[test]
fn input_vector_has_n_in_finite_values() {
    let a = standing(Role::Hero, 4.5, 1.0);
    let b = standing(Role::Monster, 9.5, -1.0);
    let x = observe(&a, &b, &arena());
    assert_eq!(x.len(), N_IN);
    assert!(x.iter().all(|v| v.is_finite()));
    assert!(x.iter().any(|v| *v != 0.0));
}

#[test]
fn inputs_stay_in_range_under_fuzzed_states() {
    let mut rng = Rng::new(99);
    for case in 0..150 {
        let mut a = Robot::new(Role::Hero);
        let mut b = Robot::new(Role::Monster);
        a.x = rng.range(0.5, 13.5);
        b.x = rng.range(0.5, 13.5);
        a.facing = if rng.f32() < 0.5 { 1.0 } else { -1.0 };
        b.facing = -a.facing;
        for r in [&mut a, &mut b] {
            for p in Part::ALL {
                if rng.f32() < 0.3 {
                    let m = r.max_hp(p);
                    r.set_hp(p, rng.range(-0.5, 1.0) * m);
                }
            }
            r.energy = rng.range(0.0, 100.0);
            r.posture = rng.range(0.0, 100.0);
            let mut t = STAND_TARGETS;
            for v in t.iter_mut() {
                *v = rng.range(-1.0, 1.0);
            }
            for _ in 0..rng.range(1.0, 40.0) as usize {
                r.apply_targets(&t, DT);
            }
        }
        let arena = if case % 2 == 0 { common::arena() } else { mech_sim::arena::Arena::generate(case as u64) };
        for (me, opp) in [(&a, &b), (&b, &a)] {
            let x = observe(me, opp, &arena);
            assert!(x.iter().all(|v| v.is_finite() && (-3.0..=3.0).contains(v)), "case {}", case);
        }
    }
}

#[test]
fn mirrored_facing_gives_identical_inputs() {
    let pose = {
        let mut t = STAND_TARGETS;
        t[Joint::ShoulderR as usize] = 0.7;
        t[Joint::ElbowL as usize] = 0.4;
        t[Joint::Waist as usize] = -0.3;
        t
    };
    let mk = |x: f32, f: f32| {
        let mut r = standing(Role::Hero, x, f);
        for _ in 0..20 {
            r.apply_targets(&pose, DT);
        }
        r
    };
    let (a, b) = (mk(4.0, 1.0), mk(9.0, -1.0));
    let (am, bm) = (mk(14.0 - 4.0, -1.0), mk(14.0 - 9.0, 1.0));
    let x1 = observe(&a, &b, &arena());
    let x2 = observe(&am, &bm, &arena());
    for (i, (p, q)) in x1.iter().zip(x2.iter()).enumerate() {
        assert!((p - q).abs() < 1e-4, "input {} differs: {} vs {}", i, p, q);
    }
}

#[test]
fn opponent_points_are_in_own_frame_forward_positive() {
    let a = standing(Role::Hero, 4.0, 1.0);
    let b = standing(Role::Monster, 9.0, -1.0);
    let x = observe(&a, &b, &arena());
    let first_x = x[OPP_POINTS.start];
    assert!(first_x > 0.0, "opponent head is in front of me (+x), got {}", first_x);
    let x2 = observe(&standing(Role::Hero, 10.0, -1.0), &standing(Role::Monster, 5.0, 1.0), &arena());
    assert!(x2[OPP_POINTS.start] > 0.0, "same when facing left");
}
