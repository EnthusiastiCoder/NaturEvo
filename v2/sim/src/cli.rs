//! CLI skeleton. `train` and `campaign` land in later workstreams.
use crate::config::ArenaConfig;
use crate::genome::Genome;
use crate::matchup::play;
use crate::world::Arena;

fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

fn need<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    opt(args, name).ok_or_else(|| format!("missing {}", name))
}
