// Assumed: arena_sim::cli::run(&[String]) -> Result<(), String> (args exclude program name).
use arena_sim::cli;
use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
use std::fs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("arena_sim_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    p
}
fn args(v: &[&str]) -> Vec<String> { v.iter().map(|s| s.to_string()).collect() }

#[test]
fn cli_train_writes_jsonl_and_champions() {
    let out = tmp("train");
    cli::run(&args(&["train", "--seed", "1", "--gens", "3", "--out", out.to_str().unwrap()])).unwrap();
    let log = fs::read_to_string(out.join("training.jsonl")).unwrap();
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines.len(), 3);
    for (i, l) in lines.iter().enumerate() {
        assert!(l.starts_with('{') && l.ends_with('}'));
        assert!(l.contains(&format!("\"gen\":{i}")));
        for k in ["win_rate_vs_hof", "best_fitness", "action_freq"] {
            assert!(l.contains(k), "missing {k}");
        }
    }
    assert!(out.join("champion_hero_gen_2.json").exists());
    assert!(out.join("champion_monster_gen_2.json").exists());
}

#[test]
fn cli_train_byte_identical_for_same_seed() {
    let go = |n: &str| {
        let out = tmp(n);
        cli::run(&args(&["train", "--seed", "4", "--gens", "2", "--out", out.to_str().unwrap()])).unwrap();
        (
            fs::read(out.join("training.jsonl")).unwrap(),
            fs::read(out.join("champion_hero_gen_1.json")).unwrap(),
        )
    };
    assert_eq!(go("det_a"), go("det_b"));
}

#[test]
fn cli_rejects_unknown_command() {
    assert!(cli::run(&args(&["bogus"])).is_err());
}

#[test]
fn cli_fight_writes_log() {
    let out = tmp("fight");
    fs::create_dir_all(&out).unwrap();
    let mut r = Rng::new(1);
    let h = out.join("h.json");
    let m = out.join("m.json");
    fs::write(&h, Genome::random(Role::Hero, &mut r).to_json()).unwrap();
    fs::write(&m, Genome::random(Role::Monster, &mut r).to_json()).unwrap();
    let f = out.join("fight.json");
    cli::run(&args(&["fight", "--hero", h.to_str().unwrap(), "--monster", m.to_str().unwrap(), "--seed", "3", "--out", f.to_str().unwrap()])).unwrap();
    assert!(fs::read_to_string(f).unwrap().contains("\"frames\""));
}

#[test]
fn trained_hero_beats_random_monsters_80_percent_of_50() {
    let mut t = Trainer::new(TrainConfig { seed: 1, generations: 40, pop: 24, k_opponents: 4, hof_sample: 2 });
    for _ in 0..40 { t.run_generation(); }
    let champ = t.hero_champion().clone();
    let cfg = ArenaConfig::default();
    let mut rng = Rng::new(777);
    let mut wins = 0;
    for s in 0..50u64 {
        let rnd = Genome::random(Role::Monster, &mut rng);
        if play(Arena::generate(s, &cfg), champ.clone(), rnd, s, &cfg).outcome == Outcome::HeroWin {
            wins += 1;
        }
    }
    assert!(wins >= 40, "beat random {wins}/50");
}
