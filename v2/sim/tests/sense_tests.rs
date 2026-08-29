// Uses assumed hooks: Match::step_with, place_for_duel(dist), face_away(Role).
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::sense::{observe, N_IN};
use arena_sim::world::Arena;

fn mk(seed: u64) -> Match {
    let cfg = ArenaConfig::default();
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    Match::new(Arena::generate(seed, &cfg), h, m, seed, &cfg)
}
fn idle() -> Action { Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false } }

#[test]
fn n_in_is_40() {
    assert_eq!(N_IN, 40);
}
