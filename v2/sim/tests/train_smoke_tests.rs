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
        for k in ["win_rate_vs_hof", "best_fitness", "action_freq", "block_unseen_share", "seen_share", "statuses_per_fight", "punish_per_fight", "draw_rate"] {
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

const GENS: u32 = 40;
// Placeholders: the WS4 implementer sets these from the real measured means (comment the numbers); the
// assertions allow measured - 0.10.
const MEASURED_BEAT_RANDOM: f64 = 0.80; // measured mean over seeds 1,2,3: TBD
const MEASURED_BEAT_GEN0: f64 = 0.80; // measured mean over seeds 1,2,3: TBD

fn rank(o: Outcome) -> u32 { match o { Outcome::HeroWin => 2, Outcome::Draw => 1, Outcome::MonsterWin => 0 } }

#[test]
fn trained_hero_beats_random_monsters_and_gen0_mean_over_three_seeds() {
    let cfg = ArenaConfig::default();
    let (mut random_mean, mut gen0_mean) = (0.0f64, 0.0f64);
    for seed in 1..=3u64 {
        let mut t = Trainer::new(TrainConfig { seed, generations: GENS, pop: 24, k_opponents: 4, hof_sample: 2 });
        t.run_generation();
        let g0 = t.hero_champion().clone();
        for _ in 1..GENS { t.run_generation(); }
        let champ = t.hero_champion().clone();
        let mon = t.monster_champion().clone();
        let mut rng = Rng::new(777 + seed);
        let (mut wr, mut better) = (0, 0);
        for s in 0..50u64 {
            let rnd = Genome::random(Role::Monster, &mut rng);
            if play(Arena::generate(s, &cfg), champ.clone(), rnd, s, &cfg).outcome == Outcome::HeroWin { wr += 1; }
            // strictly outranks gen0 on the same arena and monster; ties count as worse
            let rc = rank(play(Arena::generate(s, &cfg), champ.clone(), mon.clone(), s, &cfg).outcome);
            let rg = rank(play(Arena::generate(s, &cfg), g0.clone(), mon.clone(), s, &cfg).outcome);
            if rc > rg { better += 1; }
        }
        eprintln!("SMOKE seed {seed}: beat random {wr}/50, better than gen0 {better}/50");
        random_mean += wr as f64 / 50.0 / 3.0;
        gen0_mean += better as f64 / 50.0 / 3.0;
    }
    assert!(random_mean >= MEASURED_BEAT_RANDOM - 0.10, "mean beat-random {random_mean:.3}");
    assert!(gen0_mean >= MEASURED_BEAT_GEN0 - 0.10, "mean better-than-gen0 {gen0_mean:.3}");
}

fn eval_run(name: &str) -> std::path::PathBuf {
    let run = tmp(name);
    cli::run(&args(&["train", "--seed", "1", "--gens", "3", "--out", run.to_str().unwrap()])).unwrap();
    run
}

#[test]
fn cli_eval_writes_json_with_per_gen_role_metrics() {
    let run = eval_run("eval_ok");
    let out = run.join("report.json");
    cli::run(&args(&["eval", "--run", run.to_str().unwrap(), "--gens", "2", "--seeds", "2", "--out", out.to_str().unwrap()])).unwrap();
    let j = fs::read_to_string(&out).unwrap();
    assert!(j.starts_with('{') && j.trim_end().ends_with('}'));
    assert_eq!(j.matches('{').count(), j.matches('}').count());
    for k in ["\"version\"", "\"gens\"", "\"gen\":2", "\"hero\"", "\"monster\""] { assert!(j.contains(k), "missing {k}"); }
    for k in ["block_unseen_share", "block_share", "draw_rate", "seen_share", "statuses_per_fight", "punish_per_fight", "fights"] {
        assert!(j.contains(&format!("\"{k}\"")), "missing {k}");
    }
}

#[test]
fn cli_eval_errors_on_missing_champion_generation() {
    let run = eval_run("eval_missing");
    let out = run.join("report.json");
    cli::run(&args(&["eval", "--run", run.to_str().unwrap(), "--gens", "2", "--seeds", "2", "--out", out.to_str().unwrap()])).unwrap();
    assert!(cli::run(&args(&["eval", "--run", run.to_str().unwrap(), "--gens", "99", "--seeds", "2", "--out", out.to_str().unwrap()])).is_err());
}
