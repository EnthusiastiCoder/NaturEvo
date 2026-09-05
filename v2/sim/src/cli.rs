//! CLI skeleton. `train` and `campaign` land in later workstreams.
use crate::config::ArenaConfig;
use crate::genome::Genome;
use crate::matchup::play;
use crate::world::Arena;

fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

fn need<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    opt(args, name).ok_or_else(|| format!("missing {}", name))
}

fn load(path: &str) -> Result<Genome, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
    Genome::from_json(&s)
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some("fight") => {
            let hero = load(need(args, "--hero")?)?;
            let monster = load(need(args, "--monster")?)?;
            let seed: u64 = need(args, "--seed")?.parse().map_err(|_| "bad --seed".to_string())?;
            let out = need(args, "--out")?;
            let cfg = ArenaConfig::default();
            let res = play(Arena::generate(seed, &cfg), hero, monster, seed, &cfg);
            std::fs::write(out, res.log.to_json()).map_err(|e| format!("{}: {}", out, e))
        }
        Some("train") | Some("campaign") => Err(format!("{} not implemented yet", args[0])),
        Some(other) => Err(format!("unknown command: {}", other)),
        None => Err("usage: arena-sim <train|fight|campaign> [options]".to_string()),
    }
}
