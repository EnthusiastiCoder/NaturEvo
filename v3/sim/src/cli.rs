//! CLI skeleton. `fight`, `train`, `eval` and `compare` land in later workstreams.
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::log;

pub fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

pub fn need<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    opt(args, name).ok_or_else(|| format!("missing {}", name))
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some(other) => Err(format!("unknown command: {}", other)),
        None => Err("usage: mech-sim <fight|train|eval|compare> [options]".to_string()),
    }
}
