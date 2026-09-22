// WS4 checkpoint version tag, new GenStats columns.
use arena_sim::evolve::{GenStats, TrainConfig, Trainer};

fn cfg(seed: u64) -> TrainConfig {
    TrainConfig { seed, generations: 2, pop: 8, k_opponents: 2, hof_sample: 1 }
}
