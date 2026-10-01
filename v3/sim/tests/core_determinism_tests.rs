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

#[test]
fn rng_f32_is_top_24_bits_in_unit_interval() {
    let mut r = Rng::new(42);
    assert_eq!(r.f32(), 0.7415648698806763);
    assert_eq!(r.f32(), 0.1599103808403015);
    let mut r = Rng::new(9);
    for _ in 0..1000 {
        let v = r.f32();
        assert!((0.0..1.0).contains(&v));
        let w = r.range(-2.0, 3.0);
        assert!((-2.0..3.0).contains(&w));
    }
}

#[test]
fn rng_same_seed_same_stream_different_seed_differs() {
    let (mut a, mut b, mut c) = (Rng::new(5), Rng::new(5), Rng::new(6));
    let (va, vb, vc): (Vec<u64>, Vec<u64>, Vec<u64>) = (
        (0..20).map(|_| a.next_u64()).collect(),
        (0..20).map(|_| b.next_u64()).collect(),
        (0..20).map(|_| c.next_u64()).collect(),
    );
    assert_eq!(va, vb);
    assert_ne!(va, vc);
}

#[test]
fn sizes_are_v3_sizes() {
    assert_eq!(N_OUT, 13, "12 joint targets + drive");
    assert_eq!(N_HID, 32);
    assert!((90..=120).contains(&N_IN), "~100 inputs, got {}", N_IN);
    assert!((4000..=7000).contains(&N_PARAMS), "~5k weights, got {}", N_PARAMS);
}
