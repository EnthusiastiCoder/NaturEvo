use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::net::{Net, N_OUT};
use arena_sim::rng::Rng;
use arena_sim::sense::N_IN;

#[test]
fn n_out_is_7() {
    assert_eq!(N_OUT, 7);
}
