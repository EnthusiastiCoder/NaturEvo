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

#[test]
fn genstats_carry_behaviour_metrics_in_range() {
    let mut t = Trainer::new(cfg(1));
    t.run_generation();
    let s = &t.stats()[0];
    for i in 0..2 {
        for v in [s.block_unseen_share[i], s.seen_share[i]] { assert!((0.0..=1.0).contains(&v), "{v}"); }
        assert!(s.statuses_per_fight[i] >= 0.0 && s.punish_per_fight[i] >= 0.0);
    }
    assert!((0.0..=1.0).contains(&s.draw_rate));
}

#[test]
fn json_and_csv_expose_the_new_columns() {
    let mut t = Trainer::new(cfg(1));
    t.run_generation();
    let j = t.stats()[0].to_json();
    for k in ["block_unseen_share", "seen_share", "statuses_per_fight", "punish_per_fight", "draw_rate"] {
        assert!(j.contains(&format!("\"{k}\"")), "json missing {k}");
        assert!(GenStats::csv_header().contains(k), "csv header missing {k}");
    }
    let cols = GenStats::csv_header().split(',').count();
    assert_eq!(t.stats()[0].to_csv().split(',').count(), cols);
}
