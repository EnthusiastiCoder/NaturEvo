// Assumed helper beyond the contract: Match::step_with(hero: Action, monster: Action)
// (scripted actions, bypasses nets). Match::new takes cfg by reference.
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
