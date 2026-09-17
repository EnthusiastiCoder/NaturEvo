#![allow(dead_code)]
// Shared helpers for combat-depth tests. See v2/tests-index-combat-depth.md for the assumed hooks.
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::{Match, Setup};
use arena_sim::net::{N_OUT, N_PARAMS};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
