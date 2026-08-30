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

#[test]
fn champions_have_correct_roles_and_hof_grows() {
    let mut t = Trainer::new(cfg(2));
    t.run_generation();
    t.run_generation();
    assert_eq!(t.hero_champion().role, Role::Hero);
    assert_eq!(t.monster_champion().role, Role::Monster);
    assert!(t.hall_of_fame(Role::Hero).len() >= 2);
    assert!(t.hall_of_fame(Role::Monster).len() >= 2);
    assert!(t.hall_of_fame(Role::Hero).iter().all(|g| g.role == Role::Hero));
}

#[test]
fn same_seed_is_deterministic() {
    let run = |s| {
        let mut t = Trainer::new(cfg(s));
        t.run_generation();
        t.run_generation();
        (t.hero_champion().to_json(), t.monster_champion().to_json())
    };
    assert_eq!(run(5), run(5));
    assert_ne!(run(5), run(6));
}

#[test]
fn checkpoint_resume_matches_uninterrupted() {
    let mut a = Trainer::new(cfg(7));
    a.run_generation();
    a.run_generation();
    a.run_generation();
    let mut b = Trainer::new(cfg(7));
    b.run_generation();
    let ck = b.checkpoint();
    let mut c = Trainer::resume(&ck).unwrap();
    c.run_generation();
    c.run_generation();
    assert_eq!(a.hero_champion().to_json(), c.hero_champion().to_json());
    assert_eq!(a.stats().len(), c.stats().len());
}

#[test]
fn resume_rejects_garbage() {
    assert!(Trainer::resume("nope").is_err());
}
