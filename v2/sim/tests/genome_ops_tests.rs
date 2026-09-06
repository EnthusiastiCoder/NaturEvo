use arena_sim::fighter::Role;
use arena_sim::genome::{Genome, N_WEIGHTS};
use arena_sim::net::Net;
use arena_sim::rng::Rng;
use arena_sim::sense::N_IN;

fn g(seed: u64) -> Genome {
    Genome::random(Role::Hero, &mut Rng::new(seed))
}

#[test]
fn size_matches_layout() {
    assert_eq!(g(1).weights.len(), N_WEIGHTS);
}

#[test]
fn mutate_deterministic_and_changes() {
    let (mut a, mut b) = (g(1), g(1));
    a.mutate(&mut Rng::new(9), 0.3, 0.2);
    b.mutate(&mut Rng::new(9), 0.3, 0.2);
    assert_eq!(a.weights, b.weights);
    assert_ne!(a.weights, g(1).weights);
    assert!(a.weights.iter().all(|w| w.is_finite()));
    let mut c = g(1);
    c.mutate(&mut Rng::new(9), 1.0, 1e9);
    assert!(c.weights.iter().all(|w| w.is_finite() && w.abs() <= 8.0));
}
