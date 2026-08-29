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

#[test]
fn stats_fields_sane() {
    let mut t = Trainer::new(cfg(1));
    t.run_generation();
    let s = &t.stats()[0];
    for i in 0..2 {
        assert!(s.best_fitness[i] >= s.mean_fitness[i]);
        assert!((0.0..=1.0).contains(&s.win_rate_vs_hof[i]));
        for f in [s.action_freq[i].attack, s.action_freq[i].block, s.action_freq[i].dodge] {
            assert!((0.0..=1.0).contains(&f));
        }
    }
    assert!(s.avg_ticks > 0.0 && s.avg_ticks <= 1800.0);
}
