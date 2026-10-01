// WS0: rng / genome / net determinism (behaviour copied from v2, sizes changed). Plan sections 3 and WS0.
use mech_sim::genome::{Genome, W_CLAMP};
use mech_sim::net::{Net, N_HID, N_OUT, N_PARAMS};
use mech_sim::rng::Rng;
use mech_sim::robot::Role;
use mech_sim::sense::N_IN;

#[test]
fn rng_matches_splitmix64_reference_values() {
    let mut r = Rng::new(42);
    assert_eq!(r.next_u64(), 13679457532755275413);
    assert_eq!(r.next_u64(), 2949826092126892291);
    assert_eq!(r.next_u64(), 5139283748462763858);
}
