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

fn arr(v: &[f64]) -> String {
    let p: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
    format!("[{}]", p.join(","))
}

fn role_json(a: &Acc) -> String {
    let f = a.fights.max(1.0);
    let sum_cells = |s: usize, d: usize, k: usize| a.cells[s][d][k] / f;
    let block_unseen: f64 = (0..4).map(|d| sum_cells(0, d, 1)).sum();
    let block_all: f64 = (0..2).flat_map(|s| (0..4).map(move |d| (s, d))).map(|(s, d)| sum_cells(s, d, 1)).sum();
    let unseen_ticks: f64 = (0..4).flat_map(|d| (0..6).map(move |k| (d, k))).map(|(d, k)| sum_cells(0, d, k)).sum();
    let seen_ticks: f64 = (0..4).flat_map(|d| (0..6).map(move |k| (d, k))).map(|(d, k)| sum_cells(1, d, k)).sum();
    let far_block: f64 = (0..2).flat_map(|s| (2..4).map(move |d| (s, d))).map(|(s, d)| sum_cells(s, d, 1)).sum();
    let far_ticks: f64 = (0..2).flat_map(|s| (2..4).flat_map(move |d| (0..6).map(move |k| (s, d, k)))).map(|(s, d, k)| sum_cells(s, d, k)).sum();
    let mut action = [0.0f64; 6];
    for s in 0..2 {
        for d in 0..4 {
            for k in 0..6 {
                action[k] += sum_cells(s, d, k);
            }
        }
    }
    let kinds: Vec<f64> = a.kinds.iter().map(|x| x / a.kind_fights.max(1.0)).collect();
    let by_type: Vec<f64> = a.statuses.iter().map(|x| x / f).collect();
    let mut s = String::new();
    let _ = write!(
        s,
        "{{\"fights\":{},\"win_rate\":{},\"draw_rate\":{},\"block_unseen_share\":{},\"block_share\":{},\"seen_share\":{},\"statuses_per_fight\":{},\"statuses_by_type\":{},\"punish_per_fight\":{},\"parry_per_fight\":{},\"attack_kind_share\":{},\"block_rate_given_unseen\":{},\"block_rate_given_seen\":{},\"block_rate_given_far\":{},\"action_share\":{},\"cells\":[",
        a.fights as u64,
        a.wins / f,
        a.draws / f,
        block_unseen,
        block_all,
        seen_ticks,
        a.statuses.iter().sum::<f64>() / f,
        arr(&by_type),
        a.punish / f,
        a.parries / f,
        arr(&kinds),
        if unseen_ticks > 0.0 { block_unseen / unseen_ticks } else { 0.0 },
        if seen_ticks > 0.0 { (block_all - block_unseen) / seen_ticks } else { 0.0 },
        if far_ticks > 0.0 { far_block / far_ticks } else { 0.0 },
        arr(&action)
    );
    for si in 0..2 {
        if si > 0 {
            s.push(',');
        }
        s.push('[');
        for d in 0..4 {
            if d > 0 {
                s.push(',');
            }
            let row: Vec<f64> = (0..6).map(|k| sum_cells(si, d, k)).collect();
            s.push_str(&arr(&row));
        }
        s.push(']');
    }
    s.push_str("]}");
    s
}

/// `eval --run DIR --gens 49,99 --seeds 32 --out FILE [--panel DIR,DIR]`
pub fn eval_cli(args: &[String]) -> Result<(), String> {
    let run = need(args, "--run")?;
    let gens = parse_list(need(args, "--gens")?, "--gens")?;
    let seeds: u64 = need(args, "--seeds")?.parse().map_err(|_| "bad --seeds".to_string())?;
    let out = need(args, "--out")?;
    let panel_dirs: Vec<&str> = opt(args, "--panel").map(|p| p.split(',').collect()).unwrap_or_default();
    let mut s = format!("{{\"version\":{},\"run\":\"{}\",\"seeds\":{},\"gens\":[", EVAL_VERSION, run.replace('\\', "/"), seeds);
    for (gi, g) in gens.iter().enumerate() {
        if gi > 0 {
            s.push(',');
        }
        let _ = write!(s, "{{\"gen\":{}", g);
        for role in [Role::Hero, Role::Monster] {
            let champ = load_gen(run, role, *g)?;
            let panel = panel_for(run, &panel_dirs, role.other())?;
            let acc = aggregate(role, &champ, &panel, seeds);
            let _ = write!(s, ",\"{}\":{}", role.as_str(), role_json(&acc));
        }
        s.push('}');
    }
    s.push_str("]}");
    std::fs::write(out, s).map_err(|e| format!("{}: {}", out, e))
}

/// `compare --run DIR --late G --early G1,G2 --seeds N --out FILE [--panel DIR,DIR]`:
/// per role and early generation, fights the late champion wins and the early one loses (late), and the reverse (early).
pub fn compare_cli(args: &[String]) -> Result<(), String> {
    let run = need(args, "--run")?;
    let late: u32 = need(args, "--late")?.parse().map_err(|_| "bad --late".to_string())?;
    let early = parse_list(need(args, "--early")?, "--early")?;
    let seeds: u64 = need(args, "--seeds")?.parse().map_err(|_| "bad --seeds".to_string())?;
    let out = need(args, "--out")?;
    let panel_dirs: Vec<&str> = opt(args, "--panel").map(|p| p.split(',').collect()).unwrap_or_default();
    let cfg = ArenaConfig::default();
    let mut s = format!("{{\"version\":{},\"run\":\"{}\",\"late\":{},\"early\":[", EVAL_VERSION, run.replace('\\', "/"), late);
    let mut first = true;
    for e in &early {
        for role in [Role::Hero, Role::Monster] {
            let (lg, eg) = (load_gen(run, role, late)?, load_gen(run, role, *e)?);
            let panel = panel_for(run, &panel_dirs, role.other())?;
            let n = panel.len() * seeds as usize;
            let rows = par_map(n, |i| {
                let (o, sd) = (i / seeds as usize, 1 + (i % seeds as usize) as u64);
                let l = won(role, fight(role, &lg, &panel[o], sd, &cfg).outcome);
                let w = won(role, fight(role, &eg, &panel[o], sd, &cfg).outcome);
                (l, w)
            });
            let l = rows.iter().filter(|(l, w)| *l && !*w).count();
            let ea = rows.iter().filter(|(l, w)| !*l && *w).count();
            let both = rows.iter().filter(|(l, w)| *l && *w).count();
            let share = if l + ea == 0 { 0.5 } else { l as f64 / (l + ea) as f64 };
            if !first {
                s.push(',');
            }
            first = false;
            let _ = write!(
                s,
                "{{\"gen\":{},\"role\":\"{}\",\"late_only\":{},\"early_only\":{},\"both\":{},\"fights\":{},\"late_share\":{}}}",
                e,
                role.as_str(),
                l,
                ea,
                both,
                n,
                share
            );
        }
    }
    s.push_str("]}");
    std::fs::write(out, s).map_err(|e| format!("{}: {}", out, e))
}
