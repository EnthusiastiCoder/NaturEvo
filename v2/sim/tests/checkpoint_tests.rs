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
