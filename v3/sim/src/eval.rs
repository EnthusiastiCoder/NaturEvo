//! Held-out evaluation of a run's final champions, and the aggregate over several training seeds.
use crate::duel::{Brain, Match, Outcome};
use crate::genome::Genome;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct EvalMetrics {
    pub matches: u32,
    pub win_rate_hof: f32,
    /// (parried + blocked + dodged) / live threats, by the final champions against earlier champions.
    pub defence_rate: f32,
    pub defence_rate_scripted: f32,
    pub hit_rate: f32,
    pub idle_guard: f32,
    pub draw_share: f32,
    pub mean_ticks: f32,
    pub parry_share: f32,
    pub block_share: f32,
    pub dodge_share: f32,
}

fn champions(dir: &Path, role: &str) -> Result<Vec<Genome>, String> {
    let prefix = format!("champion_{}_gen_", role);
    let mut found: Vec<(u32, Genome)> = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {}", dir.display(), e))? {
        let name = e.map_err(|e| e.to_string())?.file_name().to_string_lossy().to_string();
        if let Some(rest) = name.strip_prefix(&prefix).and_then(|r| r.strip_suffix(".json")) {
            if let Ok(g) = rest.parse::<u32>() {
                let text = std::fs::read_to_string(dir.join(&name)).map_err(|e| e.to_string())?;
                found.push((g, Genome::from_json(&text)?));
            }
        }
    }
    found.sort_by_key(|f| f.0);
    Ok(found.into_iter().map(|f| f.1).collect())
}

#[derive(Default)]
struct Acc {
    n: u32,
    wins: u32,
    draws: u32,
    ticks: f32,
    idle: f32,
    parry: u32,
    block: u32,
    dodge: u32,
    hit: u32,
    miss: u32,
}

impl Acc {
    fn add(&mut self, r: &crate::duel::MatchResult, who: usize) {
        self.n += 1;
        let mine = if who == 0 { Outcome::WinA } else { Outcome::WinB };
        if r.outcome == mine {
            self.wins += 1;
        }
        if r.outcome == Outcome::Draw {
            self.draws += 1;
        }
        self.ticks += r.ticks as f32;
        self.idle += r.idle_guard_share[who];
        let d = &r.defence[who];
        self.parry += d.parry;
        self.block += d.block;
        self.dodge += d.dodge;
        self.hit += d.hit;
        self.miss += d.miss;
    }
    fn threats(&self) -> f32 {
        (self.parry + self.block + self.dodge + self.hit + self.miss) as f32
    }
    fn defence_rate(&self) -> f32 {
        if self.threats() > 0.0 {
            (self.parry + self.block + self.dodge) as f32 / self.threats()
        } else {
            0.0
        }
    }
}

/// Evaluate the latest hero and monster champions in `dir`.
pub fn evaluate_run(dir: &Path, arena_seeds: u64) -> Result<EvalMetrics, String> {
    let heroes = champions(dir, "hero")?;
    let mons = champions(dir, "monster")?;
    if heroes.is_empty() || mons.is_empty() {
        return Err(format!("{}: no champions", dir.display()));
    }
    let (fh, fm) = (heroes.last().unwrap().clone(), mons.last().unwrap().clone());
    let mut hof = Acc::default();
    let mut scripted = Acc::default();
    for s in 0..arena_seeds {
        let seed = 1000 + s;
        // Earlier champions (all but the final one); fall back to the final one for a single-champion run.
        let eh: Vec<&Genome> = if heroes.len() > 1 { heroes[..heroes.len() - 1].iter().collect() } else { vec![&heroes[0]] };
        let em: Vec<&Genome> = if mons.len() > 1 { mons[..mons.len() - 1].iter().collect() } else { vec![&mons[0]] };
        for m in &em {
            let r = Match::new(seed, Brain::Net(fh.clone()), Brain::Net((*m).clone())).run_to_end();
            hof.add(&r, 0);
        }
        for h in &eh {
            let r = Match::new(seed, Brain::Net((*h).clone()), Brain::Net(fm.clone())).run_to_end();
            hof.add(&r, 1);
        }
        scripted.add(&Match::new(seed, Brain::Net(fh.clone()), Brain::Scripted).run_to_end(), 0);
        scripted.add(&Match::new(seed, Brain::Scripted, Brain::Net(fm.clone())).run_to_end(), 1);
    }
    let n = hof.n.max(1) as f32;
    let def = (hof.parry + hof.block + hof.dodge).max(1) as f32;
    Ok(EvalMetrics {
        matches: hof.n,
        win_rate_hof: hof.wins as f32 / n,
        defence_rate: hof.defence_rate(),
        defence_rate_scripted: scripted.defence_rate(),
        hit_rate: if hof.threats() > 0.0 { hof.hit as f32 / hof.threats() } else { 0.0 },
        idle_guard: hof.idle / n,
        draw_share: hof.draws as f32 / n,
        mean_ticks: hof.ticks / n,
        parry_share: hof.parry as f32 / def,
        block_share: hof.block as f32 / def,
        dodge_share: hof.dodge as f32 / def,
    })
}

const KEYS: [&str; 10] = ["win_rate_hof", "defence_rate", "defence_rate_scripted", "hit_rate", "idle_guard", "draw_share", "mean_ticks", "parry_share", "block_share", "dodge_share"];

fn values(m: &EvalMetrics) -> [f32; 10] {
    [m.win_rate_hof, m.defence_rate, m.defence_rate_scripted, m.hit_rate, m.idle_guard, m.draw_share, m.mean_ticks, m.parry_share, m.block_share, m.dodge_share]
}
