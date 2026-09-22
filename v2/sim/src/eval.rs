//! Held-out evaluation of saved champions: `eval` (behaviour metrics by role and generation) and
//! `compare` (late champion against early ones on the same panel). Opponents come from other runs
//! (`--panel DIR[,DIR]`) or, if absent, from the run itself.
use crate::attacks::{Rank, Species};
use crate::config::ArenaConfig;
use crate::evolve::{par_map, species_for};
use crate::fighter::Role;
use crate::genome::Genome;
use crate::matchup::{play_with_setup, Outcome, Setup};
use crate::metrics::analyze;
use crate::world::Arena;
use std::fmt::Write;

pub const EVAL_VERSION: u32 = 1;

fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

fn need<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    opt(args, name).ok_or_else(|| format!("missing {}", name))
}

fn parse_list(s: &str, what: &str) -> Result<Vec<u32>, String> {
    s.split(',').map(|x| x.trim().parse::<u32>().map_err(|_| format!("bad {}", what))).collect()
}

fn load_gen(dir: &str, role: Role, gen: u32) -> Result<Genome, String> {
    let p = std::path::Path::new(dir).join(format!("champion_{}_gen_{}.json", role.as_str(), gen));
    let t = std::fs::read_to_string(&p).map_err(|e| format!("{}: {}", p.display(), e))?;
    Genome::from_json(&t)
}

/// Every saved champion of `role` in `dir`, ordered by generation.
fn load_all(dir: &str, role: Role) -> Result<Vec<Genome>, String> {
    let prefix = format!("champion_{}_gen_", role.as_str());
    let mut found: Vec<(u32, std::path::PathBuf)> = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {}", dir, e))? {
        let path = e.map_err(|e| e.to_string())?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if let Some(g) = name.strip_prefix(&prefix).and_then(|r| r.strip_suffix(".json")).and_then(|g| g.parse::<u32>().ok()) {
            found.push((g, path));
        }
    }
    found.sort_by_key(|f| f.0);
    found
        .iter()
        .map(|(_, p)| Genome::from_json(&std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p.display(), e))?))
        .collect()
}

fn panel_for(run: &str, panel_dirs: &[&str], opp_role: Role) -> Result<Vec<Genome>, String> {
    let dirs: Vec<&str> = if panel_dirs.is_empty() { vec![run] } else { panel_dirs.to_vec() };
    let mut out = Vec::new();
    for d in dirs {
        out.extend(load_all(d, opp_role)?);
    }
    if out.is_empty() {
        return Err("empty panel".into());
    }
    Ok(out)
}

fn fight(role: Role, me: &Genome, opp: &Genome, seed: u64, cfg: &ArenaConfig) -> crate::matchup::MatchResult {
    let (sp, rank) = species_for(seed);
    let (h, m) = if role == Role::Hero { (me.clone(), opp.clone()) } else { (opp.clone(), me.clone()) };
    let setup = Setup { hp: [cfg.max_hp; 2], species: [Species::Hero, sp], rank: [Rank::Grass, rank] };
    play_with_setup(Arena::generate(seed, cfg), h, m, seed, cfg, setup)
}

fn won(role: Role, o: Outcome) -> bool {
    matches!((o, role), (Outcome::HeroWin, Role::Hero) | (Outcome::MonsterWin, Role::Monster))
}

#[derive(Default)]
struct Acc {
    fights: f64,
    wins: f64,
    draws: f64,
    cells: [[[f64; 6]; 4]; 2],
    statuses: [f64; 7],
    punish: f64,
    parries: f64,
    kinds: [f64; 3],
    kind_fights: f64,
}

fn aggregate(role: Role, champ: &Genome, panel: &[Genome], seeds: u64) -> Acc {
    let cfg = ArenaConfig::default();
    let n = panel.len() * seeds as usize;
    let rows = par_map(n, |i| {
        let (o, s) = (i / seeds as usize, 1 + (i % seeds as usize) as u64);
        let r = fight(role, champ, &panel[o], s, &cfg);
        (analyze(&r.log, role, r.outcome), won(role, r.outcome))
    });
    let mut a = Acc::default();
    for (m, w) in &rows {
        a.fights += 1.0;
        a.wins += *w as u8 as f64;
        a.draws += m.draw as u8 as f64;
        for s in 0..2 {
            for d in 0..4 {
                for k in 0..6 {
                    a.cells[s][d][k] += m.cells[s][d][k] as f64;
                }
            }
        }
        for k in 0..7 {
            a.statuses[k] += m.statuses_applied[k] as f64;
        }
        a.punish += m.punish_hits as f64;
        a.parries += m.parries as f64;
        let sum: f32 = m.attack_kind_share.iter().sum();
        if sum > 0.0 {
            a.kind_fights += 1.0;
            for k in 0..3 {
                a.kinds[k] += m.attack_kind_share[k] as f64;
            }
        }
    }
    a
}
