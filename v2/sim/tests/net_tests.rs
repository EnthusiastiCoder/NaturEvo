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
