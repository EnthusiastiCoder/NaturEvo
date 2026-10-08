//! CLI skeleton. `fight`, `train`, `eval` and `compare` land in later workstreams.
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::log;

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

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some(other) => Err(format!("unknown command: {}", other)),
        None => Err("usage: mech-sim <fight|train|eval|compare> [options]".to_string()),
    }
}
