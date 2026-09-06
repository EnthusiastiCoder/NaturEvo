use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::play;
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn run(seed: u64) -> (String, u32) {
    let cfg = ArenaConfig::default();
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    let res = play(Arena::generate(seed, &cfg), h, m, seed, &cfg);
    (res.log.to_json(), res.ticks)
}

#[test]
fn same_seed_gives_byte_identical_log() {
    for seed in 1u64..=100 {
        assert_eq!(run(seed), run(seed));
    }
}

#[test]
fn different_seeds_differ() {
    assert_ne!(run(1).0, run(2).0);
}

#[test]
fn log_has_a_frame_per_tick_and_tick_hz() {
    let (j, ticks) = run(5);
    let n = j.matches("\"hp\"").count();
    assert!(n as u32 >= ticks, "frames {n} ticks {ticks}");
    assert!(j.contains("\"tick_hz\":30") || j.contains("\"tick_hz\": 30"));
}
