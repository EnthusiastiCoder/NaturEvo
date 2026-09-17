mod common;
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::world::Arena;

#[test]
fn spawns_13_apart_open_reachable_for_200_seeds() {
    let cfg = ArenaConfig::default();
    for seed in 0..200u64 {
        let a = Arena::generate(seed, &cfg);
        let s = a.spawns();
        assert_eq!(s[0], (17.5, 16.5), "seed {seed}");
        assert_eq!(s[1], (30.5, 16.5), "seed {seed}");
        for (x, y) in s {
            assert!(!a.is_blocked(x, y) && !a.circle_blocked(x, y, cfg.fighter_radius), "seed {seed}");
        }
        assert!(a.reachable(s[0], s[1]), "seed {seed}");
    }
}
