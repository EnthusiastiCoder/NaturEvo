//! Held-out evaluation of saved champions: `eval` (behaviour metrics by role and generation) and
//! `compare` (late champion against early ones on the same panel). Opponents come from other runs
//! (`--panel DIR[,DIR]`) or, if absent, from the run itself.
use crate::attacks::{Rank, Species};
use crate::config::ArenaConfig;
use crate::evolve::{par_map, species_for};
use crate::fighter::Role;
use crate::genome::Genome;
use crate::matchup::{play_with_setup, Outcome, Setup};
use crate::metrics::analyze;
use crate::world::Arena;
use std::fmt::Write;

pub const EVAL_VERSION: u32 = 1;
