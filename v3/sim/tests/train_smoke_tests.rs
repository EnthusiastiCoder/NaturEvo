// W4 training smoke: 3 gens, pop 8, deterministic, writes run artefacts, threaded == single thread.
use mech_sim::evolve::{train, TrainConfig};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mech_sim_test_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}
