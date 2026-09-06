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

#[test]
fn crossover_from_parents_only() {
    let (a, b) = (g(1), g(2));
    let c = Genome::crossover(&a, &b, &mut Rng::new(3));
    let d = Genome::crossover(&a, &b, &mut Rng::new(3));
    assert_eq!(c.weights, d.weights);
    assert_eq!(c.role, a.role);
    assert!(c.generation >= 1);
    for i in 0..c.weights.len() {
        assert!(c.weights[i] == a.weights[i] || c.weights[i] == b.weights[i]);
    }
    assert!(c.weights != a.weights && c.weights != b.weights);
}

#[test]
fn hidden_layer_influences_output() {
    let base = g(4);
    let x = [0.4f32; N_IN];
    let mut z = base.clone();
    let tail = z.weights.len() - 7 - 7 * 16; // start of W2
    for w in z.weights[tail..tail + 7 * 16].iter_mut() { *w = 0.0; }
    let (a, b) = (Net::from_genome(&base).forward(&x), Net::from_genome(&z).forward(&x));
    assert!(a.fwd != b.fwd || a.turn != b.turn);
}
