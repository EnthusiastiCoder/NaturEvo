// W4 training smoke: 3 gens, pop 8, deterministic, writes run artefacts, threaded == single thread.
use mech_sim::evolve::{train, TrainConfig};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mech_sim_test_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn run(name: &str, threads: usize) -> (PathBuf, String) {
    let dir = tmp(name);
    let mut cfg = TrainConfig::new(7, 8, 3, dir.clone());
    cfg.threads = threads;
    train(&cfg).expect("train");
    let csv = std::fs::read_to_string(dir.join("training.csv")).expect("training.csv");
    (dir, csv)
}

#[test]
fn writes_csv_run_json_and_champions() {
    let (dir, csv) = run("files", 1);
    assert!(csv.lines().count() >= 4, "header + 3 generations, got {}", csv.lines().count());
    assert!(dir.join("run.json").exists());
    let names: Vec<String> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
    assert!(names.iter().any(|n| n.starts_with("champion_hero_gen_") && n.ends_with(".json")), "{:?}", names);
    assert!(names.iter().any(|n| n.starts_with("champion_monster_gen_") && n.ends_with(".json")), "{:?}", names);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn same_seed_gives_identical_csv() {
    let (d1, c1) = run("det_a", 1);
    let (d2, c2) = run("det_b", 1);
    assert_eq!(c1, c2);
    let _ = std::fs::remove_dir_all(d1);
    let _ = std::fs::remove_dir_all(d2);
}

#[test]
fn threaded_evaluation_equals_single_thread() {
    let (d1, c1) = run("thr_1", 1);
    let (d4, c4) = run("thr_4", 4);
    assert_eq!(c1, c4);
    let _ = std::fs::remove_dir_all(d1);
    let _ = std::fs::remove_dir_all(d4);
}
