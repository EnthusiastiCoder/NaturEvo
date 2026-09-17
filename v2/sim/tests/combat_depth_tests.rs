// WS1 combat-depth rules. Tick convention: attack input at step 1 => hit lands on step W+1 (1-indexed).
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, CombatState as S, Role};
use arena_sim::matchup::{Match, BLOCK_HALF_ARC};
use arena_sim::status::Status;
use common::*;
use std::f32::consts::PI;
