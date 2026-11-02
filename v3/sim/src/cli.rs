//! CLI: fight, train, eval, compare.
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::log;
use crate::eval;
use crate::evolve::{train, TrainConfig};
use std::path::PathBuf;

pub fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

pub fn need<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    opt(args, name).ok_or_else(|| format!("missing {}", name))
}

fn brain(spec: &str) -> Result<Brain, String> {
    match spec {
        "dummy" => Ok(Brain::Dummy),
        "scripted" => Ok(Brain::Scripted),
        "skill1" => Ok(Brain::Skill(1)),
        "skill2" => Ok(Brain::Skill(2)),
        "skill3" => Ok(Brain::Skill(3)),
        "press" | "counter" | "duelist" => Ok(Brain::Style(crate::stylebot::StyleId::from_name(spec).unwrap())),
        path => {
            let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
            Ok(Brain::Net(Genome::from_json(&text)?))
        }
    }
}

fn fight(args: &[String]) -> Result<(), String> {
    let seed: u64 = opt(args, "--seed").unwrap_or("1").parse().map_err(|_| "bad --seed".to_string())?;
    let a = brain(opt(args, "--a").unwrap_or("scripted"))?;
    let b = brain(opt(args, "--b").or_else(|| opt(args, "--opp")).unwrap_or("dummy"))?;
    let mut m = Match::new(seed, a, b);
    m.enable_log();
    m.run_to_end();
    let json = log::fight_json(&m);
    match opt(args, "--out") {
        Some(path) => std::fs::write(path, json).map_err(|e| format!("{}: {}", path, e)),
        None => {
            println!("{}", json);
            Ok(())
        }
    }
}


fn train_cmd(args: &[String]) -> Result<(), String> {
    let seed: u64 = opt(args, "--seed").unwrap_or("1").parse().map_err(|_| "bad --seed".to_string())?;
    let pop: usize = opt(args, "--pop").unwrap_or("64").parse().map_err(|_| "bad --pop".to_string())?;
    let gens: u32 = opt(args, "--gens").unwrap_or("200").parse().map_err(|_| "bad --gens".to_string())?;
    let out = PathBuf::from(need(args, "--out")?);
    let mut cfg = TrainConfig::new(seed, pop, gens, out);
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    if let Some(st) = opt(args, "--style") {
        cfg.style = Some(crate::stylebot::StyleId::from_name(st).ok_or("bad --style")?);
    }
    if let Some(d) = opt(args, "--seed-dir") {
        cfg.seed_genome = Some(eval::final_champions(std::path::Path::new(d))?.0);
    }
    if let Some(f) = opt(args, "--foes") {
        for p in f.split(',') {
            let t = std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p, e))?;
            cfg.foes.push(Genome::from_json(&t)?);
        }
    }
    cfg.threads = opt(args, "--threads").map(|t| t.parse().unwrap_or(1)).unwrap_or(cores);
    let t = std::time::Instant::now();
    train(&cfg)?;
    eprintln!("trained {} gens pop {} in {:.1}s on {} threads", gens, pop, t.elapsed().as_secs_f64(), cfg.threads);
    Ok(())
}

fn eval_cmd(args: &[String]) -> Result<(), String> {
    let runs: Vec<String> = need(args, "--runs")?.split(',').map(|s| s.to_string()).collect();
    let seeds: u64 = opt(args, "--seeds").unwrap_or("4").parse().map_err(|_| "bad --seeds".to_string())?;
    let mut ms = Vec::new();
    let mut names = Vec::new();
    for r in &runs {
        let p = std::path::Path::new(r);
        ms.push(eval::evaluate_run(p, seeds)?);
        names.push(p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| r.clone()));
    }
    let md = eval::aggregate_markdown(&names, &ms);
    println!("{}", md);
    if let Some(p) = opt(args, "--out") {
        std::fs::write(p, eval::aggregate_json(&names, &ms)).map_err(|e| format!("{}: {}", p, e))?;
    }
    if let Some(p) = opt(args, "--md") {
        std::fs::write(p, md).map_err(|e| format!("{}: {}", p, e))?;
    }
    Ok(())
}

/// Smoothness diagnosis of a run: hero/monster champions per saved generation vs the scripted attacker.
fn diag_cmd(args: &[String]) -> Result<(), String> {
    use crate::diag::*;
    let dir = PathBuf::from(need(args, "--run")?);
    let seeds: u64 = opt(args, "--seeds").unwrap_or("3").parse().map_err(|_| "bad --seeds".to_string())?;
    let mut gens: Vec<u32> = Vec::new();
    for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let n = e.map_err(|e| e.to_string())?.file_name().to_string_lossy().to_string();
        if let Some(g) = n.strip_prefix("champion_hero_gen_").and_then(|r| r.strip_suffix(".json")).and_then(|r| r.parse::<u32>().ok()) {
            gens.push(g);
        }
    }
    gens.sort();
    let load = |role: &str, g: u32| -> Result<Genome, String> {
        let p = dir.join(format!("champion_{}_gen_{}.json", role, g));
        Genome::from_json(&std::fs::read_to_string(&p).map_err(|e| format!("{}: {}", p.display(), e))?)
    };
    let mut out = String::from(TABLE_HEAD);
    let step = (gens.len() / 6).max(1);
    for (k, g) in gens.iter().enumerate() {
        if k % step != 0 && k + 1 != gens.len() {
            continue;
        }
        let (h, m) = (Brain::Net(load("hero", *g)?), Brain::Net(load("monster", *g)?));
        let mut v = Vec::new();
        for s in 0..seeds {
            let (sm, _) = measure(h.clone(), m.clone(), 1000 + s);
            v.push(sm[0]);
            v.push(sm[1]);
        }
        out.push_str(&row(&format!("net gen {} (hero+monster)", g), &mean_smooth(&v)));
    }
    let mut v = Vec::new();
    for s in 0..seeds.max(6) {
        let (sm, _) = measure(Brain::Scripted, Brain::Dummy, 1000 + s);
        v.push(sm[0]);
    }
    out.push_str(&row("scripted attacker", &mean_smooth(&v)));
    print!("{}", out);
    Ok(())
}

fn compare_cmd(args: &[String]) -> Result<(), String> {
    let a = brain(need(args, "--a")?)?;
    let b = brain(need(args, "--b")?)?;
    let n: u64 = opt(args, "--seeds").unwrap_or("10").parse().map_err(|_| "bad --seeds".to_string())?;
    let (mut wa, mut wb, mut d) = (0, 0, 0);
    for s in 0..n {
        match Match::new(1000 + s, a.clone(), b.clone()).run_to_end().outcome {
            crate::duel::Outcome::WinA => wa += 1,
            crate::duel::Outcome::WinB => wb += 1,
            crate::duel::Outcome::Draw => d += 1,
        }
    }
    println!("a wins {} b wins {} draws {} of {}", wa, wb, d, n);
    Ok(())
}

/// Style matrix. `--teachers` adds the three scripted teachers; `--dirs press=DIR,counter=DIR,...` adds trained styles
/// (hero champion in slot 0, monster champion in slot 1, latest generation). `--flat` removes arena obstacles.
fn styles_cmd(args: &[String]) -> Result<(), String> {
    use crate::stylematrix::*;
    let seeds: u64 = opt(args, "--seeds").unwrap_or("8").parse().map_err(|_| "bad --seeds".to_string())?;
    let flat = args.iter().any(|a| a == "--flat");
    let mut cs: Vec<Contestant> = Vec::new();
    if args.iter().any(|a| a == "--teachers") {
        for st in crate::stylebot::StyleId::ALL {
            cs.push(Contestant { name: format!("teacher_{}", st.name()), as_a: Brain::Style(st), as_b: Brain::Style(st) });
        }
    }
    if let Some(d) = opt(args, "--dirs") {
        for kv in d.split(',') {
            let (name, dir) = kv.split_once('=').ok_or("--dirs needs name=dir")?;
            let (h, m) = eval::final_champions(std::path::Path::new(dir))?;
            cs.push(Contestant { name: name.to_string(), as_a: Brain::Net(h), as_b: Brain::Net(m) });
        }
    }
    if let Some(d) = opt(args, "--extra") {
        for n in d.split(',') {
            cs.push(Contestant { name: n.to_string(), as_a: brain(n)?, as_b: brain(n)? });
        }
    }
    let m = matrix(&cs, seeds, flat);
    let title = format!("Style matrix ({} seeds per ordered pair{})", seeds, if flat { ", flat arena" } else { "" });
    let md = markdown(&title, &m);
    println!("{}", md);
    if let Some(p) = opt(args, "--md") {
        std::fs::write(p, md).map_err(|e| format!("{}: {}", p, e))?;
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some("fight") => fight(&args[1..]),
        Some("train") => train_cmd(&args[1..]),
        Some("eval") => eval_cmd(&args[1..]),
        Some("diag") => diag_cmd(&args[1..]),
        Some("capdiag") => {
            let runs: Vec<String> = need(&args[1..], "--runs")?.split(',').map(|s| s.to_string()).collect();
            let seeds: u64 = opt(&args[1..], "--seeds").unwrap_or("4").parse().map_err(|_| "bad --seeds".to_string())?;
            print!("{}", crate::capdiag::capdiag(&runs, seeds)?);
            Ok(())
        }
        Some("distil") => {
            let a = &args[1..];
            let style = crate::stylebot::StyleId::from_name(need(a, "--style")?).ok_or("bad --style")?;
            let mut o = crate::imitate::DistilOpts::default();
            let num = |n: &str, d: usize| -> usize { opt(a, n).and_then(|v| v.parse().ok()).unwrap_or(d) };
            o.per_opp = num("--per-opp", o.per_opp);
            o.rounds = num("--rounds", o.rounds);
            o.epochs = num("--epochs", o.epochs);
            o.seed = num("--seed", 1) as u64;
            let t = std::time::Instant::now();
            let (g, st) = crate::imitate::distil(style, &o);
            eprintln!("distilled {} in {:.1}s: loss {:.4} move_acc {:.3} active_acc {:.3} on {} samples", style.name(), t.elapsed().as_secs_f64(), st.loss, st.move_acc, st.active_acc, st.samples);
            // --out DIR: champion_hero_gen_0.json and champion_monster_gen_0.json (the drive-mirrored variant).
            let out = std::path::PathBuf::from(need(a, "--out")?);
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            let hero = g.mirrored_drive(crate::robot::Role::Hero);
            let mon = g.mirrored_drive(crate::robot::Role::Monster);
            std::fs::write(out.join("champion_hero_gen_0.json"), hero.to_json()).map_err(|e| e.to_string())?;
            std::fs::write(out.join("champion_monster_gen_0.json"), mon.to_json()).map_err(|e| e.to_string())
        }
        Some("styles") => styles_cmd(&args[1..]),
        Some("compare") => compare_cmd(&args[1..]),
        Some(other) => Err(format!("unknown command: {}", other)),
        None => Err("usage: mech-sim <fight|train|eval|compare> [options]".to_string()),
    }
}
