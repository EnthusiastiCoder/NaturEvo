// Plan section 4 (Phase 1 arena): flat 14 m, 2-3 seeded obstacles 0.6-1.5 m high.
use mech_sim::arena::Arena;

#[test]
fn flat_has_width_and_no_obstacles() {
    let a = Arena::flat(14.0);
    assert_eq!(a.width, 14.0);
    assert!(a.obstacles.is_empty());
}
