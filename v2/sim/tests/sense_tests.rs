// Uses assumed hooks: Match::step_with, place_for_duel(dist), face_away(Role).
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::sense::{observe, N_IN};
use arena_sim::world::Arena;
