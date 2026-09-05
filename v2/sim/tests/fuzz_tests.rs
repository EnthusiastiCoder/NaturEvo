// WS2 extras: fuzz (no panics) and release-mode throughput.
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Match};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
