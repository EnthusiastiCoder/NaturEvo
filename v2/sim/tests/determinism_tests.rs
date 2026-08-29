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
