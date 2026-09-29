// Plan section 4 (Phase 1 arena): flat 14 m, 2-3 seeded obstacles 0.6-1.5 m high.
use mech_sim::arena::Arena;

#[test]
fn flat_has_width_and_no_obstacles() {
    let a = Arena::flat(14.0);
    assert_eq!(a.width, 14.0);
    assert!(a.obstacles.is_empty());
}

#[test]
fn generate_is_deterministic_and_in_spec() {
    let a = Arena::generate(5);
    let b = Arena::generate(5);
    assert_eq!(a.width, 14.0);
    assert_eq!(a.obstacles.len(), b.obstacles.len());
    assert!((2..=3).contains(&a.obstacles.len()));
    for (o, p) in a.obstacles.iter().zip(&b.obstacles) {
        assert_eq!((o.x, o.w, o.h), (p.x, p.w, p.h));
        assert!((0.6..=1.5).contains(&o.h), "h {}", o.h);
        assert!(o.w > 0.0 && o.x - o.w / 2.0 >= 0.0 && o.x + o.w / 2.0 <= 14.0);
    }
}

#[test]
fn different_seeds_give_different_arenas() {
    let sig = |s: u64| Arena::generate(s).obstacles.iter().map(|o| (o.x.to_bits(), o.h.to_bits())).collect::<Vec<_>>();
    assert!((1..20u64).any(|s| sig(s) != sig(0)));
}
