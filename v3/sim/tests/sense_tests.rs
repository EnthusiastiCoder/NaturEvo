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
