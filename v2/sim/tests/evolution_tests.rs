// Assumed: Trainer::checkpoint(&self) -> String, Trainer::resume(&str) -> Result<Trainer, String>.
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;

fn cfg(seed: u64) -> TrainConfig {
    TrainConfig { seed, generations: 3, pop: 8, k_opponents: 2, hof_sample: 1 }
}
