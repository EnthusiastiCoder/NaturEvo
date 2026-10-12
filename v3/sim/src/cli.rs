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
        path => {
            let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
            Ok(Brain::Net(Genome::from_json(&text)?))
        }
    }
}

fn fight(args: &[String]) -> Result<(), String> {
    let seed: u64 = opt(args, "--seed").unwrap_or("1").parse().map_err(|_| "bad --seed".to_string())?;
    let a = brain(opt(args, "--a").unwrap_or("scripted"))?;
    let b = brain(opt(args, "--b").unwrap_or("dummy"))?;
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

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some("fight") => fight(&args[1..]),
        Some("train") => train_cmd(&args[1..]),
        Some("eval") => eval_cmd(&args[1..]),
        Some("compare") => compare_cmd(&args[1..]),
        Some(other) => Err(format!("unknown command: {}", other)),
        None => Err("usage: mech-sim <fight|train|eval|compare> [options]".to_string()),
    }
}
