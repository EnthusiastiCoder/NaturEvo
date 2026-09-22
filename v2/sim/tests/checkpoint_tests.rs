// WS4 checkpoint version tag, new GenStats columns.
use arena_sim::evolve::{GenStats, TrainConfig, Trainer};

fn cfg(seed: u64) -> TrainConfig {
    TrainConfig { seed, generations: 2, pop: 8, k_opponents: 2, hof_sample: 1 }
}

fn err_of(text: &str) -> String {
    match Trainer::resume(text) { Ok(_) => panic!("resume unexpectedly succeeded"), Err(e) => e }
}

#[test]
fn checkpoint_header_is_v2() {
    let mut t = Trainer::new(cfg(7));
    t.run_generation();
    assert!(t.checkpoint().starts_with("arena-checkpoint-v2\n"));
}

#[test]
fn v1_checkpoint_is_rejected_with_a_version_error() {
    let mut t = Trainer::new(cfg(7));
    t.run_generation();
    let v1 = t.checkpoint().replacen("arena-checkpoint-v2", "arena-checkpoint-v1", 1);
    let e = err_of(&v1);
    assert!(e.contains("version"), "error was: {e}");
}

#[test]
fn v2_checkpoint_round_trips() {
    let mut a = Trainer::new(cfg(8));
    a.run_generation();
    let mut b = Trainer::resume(&a.checkpoint()).unwrap();
    a.run_generation();
    b.run_generation();
    assert_eq!(a.hero_champion().to_json(), b.hero_champion().to_json());
    assert_eq!(a.stats().len(), b.stats().len());
}
