// Assumed: arena_sim::cli::run(&[String]) -> Result<(), String> (args exclude program name).
use arena_sim::cli;
use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
use std::fs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("arena_sim_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    p
}
fn args(v: &[&str]) -> Vec<String> { v.iter().map(|s| s.to_string()).collect() }
