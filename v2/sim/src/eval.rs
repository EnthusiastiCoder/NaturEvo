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

fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

fn need<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    opt(args, name).ok_or_else(|| format!("missing {}", name))
}

fn parse_list(s: &str, what: &str) -> Result<Vec<u32>, String> {
    s.split(',').map(|x| x.trim().parse::<u32>().map_err(|_| format!("bad {}", what))).collect()
}
