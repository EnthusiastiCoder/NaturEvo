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
