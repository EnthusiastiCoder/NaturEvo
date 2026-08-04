//! Walking hero, encounters, elites, moves, determinism and cap tests
//! (plan 2026-10-21_hero-map-walking, workstream 2). Written from the plan before any implementation.
//!
//! Events are read through run_body() JSON (parsed by tests/common) so only these Rust names are assumed:
//!   Run.steps (counter), Summary.steps, Config fields from the plan, jungle_sim::map::Map (see map_tests.rs).
//! The map of a run is assumed to be Map::generate(seed, cfg.map_size).
mod common;
use common::{parse, J};
use jungle_sim::config::Config;
use jungle_sim::data::{stage_name, MAX_STAGE, MOVES, SPECIES, ZONES};
use jungle_sim::map::{Map, Tile};
use jungle_sim::report::{batch_with, run_body, run_window_body};
use jungle_sim::sim::{simulate_summary, simulate_with, Run};
use std::collections::{HashMap, HashSet};
