// Assumed helper beyond the contract: Match::step_with(hero: Action, monster: Action)
// (scripted actions, bypasses nets). Match::new takes cfg by reference.
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn mk(seed: u64) -> (Match, ArenaConfig) {
    let cfg = ArenaConfig::default();
    let arena = Arena::generate(seed, &cfg);
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    (Match::new(arena, h, m, seed, &cfg), cfg)
}
fn idle() -> Action { Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false } }
