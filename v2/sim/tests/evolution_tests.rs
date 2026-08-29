// Assumed: Trainer::checkpoint(&self) -> String, Trainer::resume(&str) -> Result<Trainer, String>.
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;

fn cfg(seed: u64) -> TrainConfig {
    TrainConfig { seed, generations: 3, pop: 8, k_opponents: 2, hof_sample: 1 }
}

#[test]
fn run_generation_appends_stats() {
    let mut t = Trainer::new(cfg(1));
    assert_eq!(t.stats().len(), 0);
    t.run_generation();
    t.run_generation();
    assert_eq!(t.stats().len(), 2);
    assert_eq!(t.stats()[0].gen, 0);
    assert_eq!(t.stats()[1].gen, 1);
}
