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

fn load_gen(dir: &str, role: Role, gen: u32) -> Result<Genome, String> {
    let p = std::path::Path::new(dir).join(format!("champion_{}_gen_{}.json", role.as_str(), gen));
    let t = std::fs::read_to_string(&p).map_err(|e| format!("{}: {}", p.display(), e))?;
    Genome::from_json(&t)
}

/// Every saved champion of `role` in `dir`, ordered by generation.
fn load_all(dir: &str, role: Role) -> Result<Vec<Genome>, String> {
    let prefix = format!("champion_{}_gen_", role.as_str());
    let mut found: Vec<(u32, std::path::PathBuf)> = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {}", dir, e))? {
        let path = e.map_err(|e| e.to_string())?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if let Some(g) = name.strip_prefix(&prefix).and_then(|r| r.strip_suffix(".json")).and_then(|g| g.parse::<u32>().ok()) {
            found.push((g, path));
        }
    }
    found.sort_by_key(|f| f.0);
    found
        .iter()
        .map(|(_, p)| Genome::from_json(&std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p.display(), e))?))
        .collect()
}
