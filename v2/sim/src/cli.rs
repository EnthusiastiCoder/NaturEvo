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
        Some("train") => crate::evolve::train_cli(args),
        Some("campaign") => {
            let dir = need(args, "--champions")?;
            let out = need(args, "--out")?;
            let seed: u64 = opt(args, "--seed").unwrap_or("1").parse().map_err(|_| "bad --seed".to_string())?;
            let keep: usize = opt(args, "--keep").unwrap_or("4").parse().map_err(|_| "bad --keep".to_string())?;
            let permadeath = opt(args, "--permadeath").map(|v| v != "off").unwrap_or(true);
            let heroes = crate::campaign::Library::from_dir(dir, "hero", keep)?;
            let monsters = crate::campaign::Library::from_dir(dir, "monster", keep)?;
            let r = crate::campaign::run(seed, crate::campaign::MapCfg::default(), &heroes, &monsters, permadeath);
            let fights = opt(args, "--fights-dir");
            let json = crate::campaign::run_json(&r, fights, Some(out))?;
            std::fs::write(out, json).map_err(|e| format!("{}: {}", out, e))?;
            let won = r.encounters.iter().filter(|e| e.result.outcome == crate::matchup::Outcome::HeroWin).count();
            eprintln!("outcome {:?} steps {} encounters {} hero_wins {}", r.outcome, r.steps, r.encounters.len(), won);
            Ok(())
        }
        Some(other) => Err(format!("unknown command: {}", other)),
        None => Err("usage: arena-sim <train|fight|campaign> [options]".to_string()),
    }
}
