// WS2: N_OUT 9, N_HID 24, attack-kind argmax over outputs 6..=8 (ties to the lowest).
mod common;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::net::{Net, N_HID, N_OUT, N_PARAMS};
use arena_sim::rng::Rng;
use arena_sim::sense::N_IN;
use common::bias_genome;

#[test]
fn sizes_and_param_formula() {
    assert_eq!(N_OUT, 9);
    assert_eq!(N_HID, 24);
    assert_eq!(N_PARAMS, N_HID * N_IN + N_HID * N_HID + N_HID + N_OUT * N_HID + N_OUT);
    assert_eq!(N_PARAMS, 2745);
}

#[test]
fn random_genome_is_seeded() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let b = Genome::random(Role::Hero, &mut Rng::new(1));
    let c = Genome::random(Role::Hero, &mut Rng::new(2));
    assert_eq!(a.weights, b.weights);
    assert_ne!(a.weights, c.weights);
    assert_eq!(a.weights.len(), N_PARAMS);
    assert!(a.weights.iter().all(|w| w.is_finite()));
}

#[test]
fn json_roundtrip_exact() {
    let mut g = Genome::random(Role::Monster, &mut Rng::new(5));
    g.generation = 17;
    g.fitness = 3.5;
    let back = Genome::from_json(&g.to_json()).unwrap();
    assert_eq!(back.weights, g.weights);
    assert_eq!(back.generation, 17);
    assert_eq!(back.fitness, 3.5);
    assert_eq!(back.role, Role::Monster);
    assert_eq!(back.to_json(), g.to_json());
}

#[test]
fn from_json_rejects_garbage() {
    assert!(Genome::from_json("not json").is_err());
    assert!(Genome::from_json("{}").is_err());
}

#[test]
fn forward_is_deterministic_and_bounded() {
    let g = Genome::random(Role::Hero, &mut Rng::new(3));
    let mut n1 = Net::from_genome(&g);
    let mut n2 = Net::from_genome(&g);
    let mut r = Rng::new(10);
    for _ in 0..100 {
        let mut x = [0.0f32; N_IN];
        for v in x.iter_mut() { *v = r.range(-1.0, 1.0); }
        let a = n1.forward(&x);
        let b = n2.forward(&x);
        assert_eq!(
            (a.fwd, a.strafe, a.turn, a.attack, a.block, a.dodge, a.attack_kind),
            (b.fwd, b.strafe, b.turn, b.attack, b.block, b.dodge, b.attack_kind)
        );
        for v in [a.fwd, a.strafe, a.turn] {
            assert!((-1.0..=1.0).contains(&v));
        }
        assert!(a.attack_kind <= 2);
    }
}

#[test]
fn different_weights_give_different_actions() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let b = Genome::random(Role::Hero, &mut Rng::new(2));
    let x = [0.5f32; N_IN];
    let (mut na, mut nb) = (Net::from_genome(&a), Net::from_genome(&b));
    let (oa, ob) = (na.forward(&x), nb.forward(&x));
    assert!(oa.fwd != ob.fwd || oa.strafe != ob.strafe || oa.turn != ob.turn);
}

#[test]
fn reset_restores_initial_behaviour() {
    let g = Genome::random(Role::Hero, &mut Rng::new(4));
    let mut n = Net::from_genome(&g);
    let x = [0.3f32; N_IN];
    let first = n.forward(&x);
    for _ in 0..20 { n.forward(&[-0.7f32; N_IN]); }
    n.reset();
    let again = n.forward(&x);
    assert_eq!((first.fwd, first.turn), (again.fwd, again.turn));
}

fn kind(bias: &[(usize, f32)]) -> u8 {
    let mut n = Net::from_genome(&bias_genome(Role::Hero, bias));
    n.forward(&[0.0f32; N_IN]).attack_kind
}

#[test]
fn attack_kind_is_argmax_of_outputs_6_to_8() {
    assert_eq!(kind(&[(6, 0.8)]), 0);
    assert_eq!(kind(&[(7, 0.8)]), 1);
    assert_eq!(kind(&[(8, 0.8)]), 2);
    assert_eq!(kind(&[(6, -0.5), (7, -0.9), (8, -0.2)]), 2);
    assert_eq!(kind(&[(6, 0.3), (7, 0.9), (8, 0.5)]), 1);
}

#[test]
fn attack_kind_ties_pick_lowest() {
    assert_eq!(kind(&[]), 0, "all equal -> light");
    assert_eq!(kind(&[(6, 0.5), (7, 0.5)]), 0);
    assert_eq!(kind(&[(7, 0.5), (8, 0.5)]), 1);
    assert_eq!(kind(&[(6, 0.5), (7, 0.1), (8, 0.5)]), 0);
}

#[test]
fn same_genome_same_actions_over_a_sequence() {
    let g = Genome::random(Role::Monster, &mut Rng::new(8));
    let (mut a, mut b) = (Net::from_genome(&g), Net::from_genome(&g));
    let mut r = Rng::new(2);
    for _ in 0..50 {
        let mut x = [0.0f32; N_IN];
        for v in x.iter_mut() { *v = r.range(-1.0, 1.0); }
        let (p, q) = (a.forward(&x), b.forward(&x));
        assert_eq!((p.fwd, p.attack, p.block, p.attack_kind), (q.fwd, q.attack, q.block, q.attack_kind));
    }
}
