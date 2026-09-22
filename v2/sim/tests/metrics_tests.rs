// WS4 metrics. API (see v2/tests-index-ws46.md): metrics::analyze(&FightLog, Role, Outcome) -> FightMetrics
// { cells: [[[f32;6];4];2] indexed [seen][dist bucket][action], statuses_applied: [u32;7], punish_hits: u32,
//   parries: u32, attack_kind_share: [f32;3], draw: bool, decisive: bool }, FightMetrics::block_unseen_share() -> f32.
// Dist buckets: 0 close<=1.2, 1 mid<=4, 2 far<=12, 3 vfar. Actions: 0 ATTACK 1 BLOCK 2 DODGE 3 MOVE 4 IDLE 5 INVOLUNTARY.
// Action classes follow the fighter's intent bits; INVOLUNTARY = ticks in Staggered/Stunned/Knocked.
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::matchup::{play_with_setup, MatchResult, Setup};
use arena_sim::metrics::analyze;
use arena_sim::world::Arena;
use common::*;
