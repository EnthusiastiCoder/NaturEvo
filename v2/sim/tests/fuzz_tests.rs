// WS2 extras: fuzz (no panics) and release-mode throughput.
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Match};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

#[test]
fn fuzz_random_and_extreme_actions_never_panic() {
    let cfg = ArenaConfig::default();
    for seed in 0..40u64 {
        let mut r = Rng::new(seed);
        let h = Genome::random(Role::Hero, &mut r);
        let m = Genome::random(Role::Monster, &mut r);
        let mut mt = Match::new(Arena::generate(seed, &cfg), h, m, seed, &cfg);
        if seed % 3 == 0 {
            mt.place_for_duel(r.range(0.5, 6.0));
        }
        for _ in 0..cfg.max_ticks {
            let mut act = || Action {
                fwd: [r.range(-3.0, 3.0), f32::NAN, f32::INFINITY, 0.0][(r.next_u64() % 4) as usize],
                strafe: r.range(-3.0, 3.0),
                turn: r.range(-3.0, 3.0),
                attack: r.f32() < 0.4,
                block: r.f32() < 0.3,
                dodge: r.f32() < 0.2,
            };
            let (a, b) = (act(), act());
            mt.step_with(a, b);
            for role in [Role::Hero, Role::Monster] {
                let (x, y) = mt.pos(role);
                assert!(x.is_finite() && y.is_finite());
                assert!(mt.hp(role) >= 0.0 && mt.stamina(role) >= 0.0);
            }
        }
    }
}
