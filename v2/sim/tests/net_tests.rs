use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::net::{Net, N_OUT};
use arena_sim::rng::Rng;
use arena_sim::sense::N_IN;

#[test]
fn n_out_is_7() {
    assert_eq!(N_OUT, 7);
}

#[test]
fn random_genome_is_seeded() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let b = Genome::random(Role::Hero, &mut Rng::new(1));
    let c = Genome::random(Role::Hero, &mut Rng::new(2));
    assert_eq!(a.weights, b.weights);
    assert_ne!(a.weights, c.weights);
    assert!(!a.weights.is_empty());
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
            (a.fwd, a.strafe, a.turn, a.attack, a.block, a.dodge),
            (b.fwd, b.strafe, b.turn, b.attack, b.block, b.dodge)
        );
        for v in [a.fwd, a.strafe, a.turn] {
            assert!((-1.0..=1.0).contains(&v));
        }
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
