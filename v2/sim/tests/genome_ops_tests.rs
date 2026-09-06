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
