// WS3 anti-degenerate fitness. Hooks: evolve::{score_result, terms}, Match::{result, apply_status, set_hp, face_away}.
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{score_result, terms};
use arena_sim::fighter::{Action, Role};
use arena_sim::matchup::{Match, MatchResult};
use arena_sim::status::Status;
use common::*;

fn cfg() -> ArenaConfig { ArenaConfig::default() }
