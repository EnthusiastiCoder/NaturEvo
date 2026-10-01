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

#[test]
fn random_genome_is_seeded_and_sized() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let b = Genome::random(Role::Hero, &mut Rng::new(1));
    let c = Genome::random(Role::Hero, &mut Rng::new(2));
    assert_eq!(a.weights, b.weights);
    assert_ne!(a.weights, c.weights);
    assert_eq!(a.weights.len(), N_PARAMS);
    assert!(a.weights.iter().all(|w| w.is_finite()));
}

#[test]
fn mutate_rate_zero_is_identity_and_clamps() {
    let base = Genome::random(Role::Hero, &mut Rng::new(3));
    let mut same = base.clone();
    same.mutate(&mut Rng::new(1), 0.0, 0.5);
    assert_eq!(same.weights, base.weights);
    let mut wild = base.clone();
    wild.mutate(&mut Rng::new(1), 1.0, 1000.0);
    assert!(wild.weights.iter().all(|w| w.abs() <= W_CLAMP));
    let (mut m1, mut m2) = (base.clone(), base.clone());
    m1.mutate(&mut Rng::new(77), 0.3, 0.1);
    m2.mutate(&mut Rng::new(77), 0.3, 0.1);
    assert_eq!(m1.weights, m2.weights);
    assert_ne!(m1.weights, base.weights);
}

#[test]
fn crossover_takes_each_gene_from_a_parent_and_is_seeded() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let mut b = Genome::random(Role::Hero, &mut Rng::new(2));
    b.generation = 4;
    let c1 = Genome::crossover(&a, &b, &mut Rng::new(9));
    let c2 = Genome::crossover(&a, &b, &mut Rng::new(9));
    assert_eq!(c1.weights, c2.weights);
    assert_eq!(c1.weights.len(), N_PARAMS);
    assert!(c1.weights.iter().enumerate().all(|(i, w)| *w == a.weights[i] || *w == b.weights[i]));
    assert!(c1.weights.iter().enumerate().any(|(i, w)| *w == a.weights[i] && a.weights[i] != b.weights[i]));
    assert!(c1.weights.iter().enumerate().any(|(i, w)| *w == b.weights[i] && a.weights[i] != b.weights[i]));
    assert_eq!(c1.generation, 5, "max(parent generations) + 1");
}

#[test]
fn json_roundtrip_exact_and_rejects_garbage() {
    let mut g = Genome::random(Role::Monster, &mut Rng::new(5));
    g.generation = 17;
    g.fitness = 3.5;
    let back = Genome::from_json(&g.to_json()).unwrap();
    assert_eq!(back.weights, g.weights);
    assert_eq!((back.generation, back.fitness, back.role), (17, 3.5, Role::Monster));
    assert_eq!(back.to_json(), g.to_json());
    assert!(Genome::from_json("not json").is_err());
    assert!(Genome::from_json("{}").is_err());
}

#[test]
fn forward_is_deterministic_bounded_and_13_wide() {
    let g = Genome::random(Role::Hero, &mut Rng::new(3));
    let (mut n1, mut n2) = (Net::from_genome(&g), Net::from_genome(&g));
    let mut r = Rng::new(10);
    for _ in 0..100 {
        let mut x = [0.0f32; N_IN];
        for v in x.iter_mut() { *v = r.range(-1.0, 1.0); }
        let (a, b) = (n1.forward(&x), n2.forward(&x));
        assert_eq!(a.len(), 13);
        assert_eq!(a, b);
        assert!(a.iter().all(|v| v.is_finite() && (-1.0..=1.0).contains(v)));
    }
}

#[test]
fn recurrent_state_matters_and_reset_restores_behaviour() {
    let g = Genome::random(Role::Hero, &mut Rng::new(4));
    let mut n = Net::from_genome(&g);
    let x = [0.3f32; N_IN];
    let first = n.forward(&x);
    for _ in 0..20 { n.forward(&[-0.7f32; N_IN]); }
    let drifted = n.forward(&x);
    assert_ne!(first, drifted, "context units carry state");
    n.reset();
    assert_eq!(n.forward(&x), first);
}

#[test]
fn different_weights_give_different_outputs() {
    let a = Genome::random(Role::Hero, &mut Rng::new(1));
    let b = Genome::random(Role::Hero, &mut Rng::new(2));
    let x = [0.5f32; N_IN];
    assert_ne!(Net::from_genome(&a).forward(&x), Net::from_genome(&b).forward(&x));
}
