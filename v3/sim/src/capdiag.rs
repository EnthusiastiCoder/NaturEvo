//! Measurement of what trained nets do in capped fights (diagnostics only, never read by fitness).
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::moveset::MoveId;
use crate::skillbot::is_attack;
use std::path::Path;

#[derive(Default, Clone, Debug)]
pub struct CapStats {
    pub fights: u32,
    pub capped: u32,
    /// Sums over capped fights (per fighter 0 and 1).
    pub gap_sum: f64,
    pub gap_n: f64,
    pub swings: [f64; 2],
    pub parries: [f64; 2],
    pub guards: [f64; 2],
    pub idles: [f64; 2],
    pub ticks: f64,
    /// Fights where fighter 0 / 1 made the first attack swing.
    pub first: [u32; 2],
    pub no_attack: u32,
}

/// Play one fight and fold it into `s` if capped (or into `all` for every fight).
pub fn play(a: Brain, b: Brain, seed: u64, s: &mut CapStats, all: &mut CapStats) {
    let mut m = Match::new(seed, a, b);
    m.disable_grace_trace();
    let mut gap = 0.0f64;
    let mut n = 0.0f64;
    let mut seen = [0usize; 2];
    let mut first: Option<usize> = None;
    while !m.is_over() {
        m.step();
        gap += (m.robots[0].x - m.robots[1].x).abs() as f64;
        n += 1.0;
        if first.is_none() {
            for i in 0..2 {
                let st = m.move_starts(i);
                if st.len() > seen[i] {
                    if st[seen[i]..].iter().any(|&id| is_attack(id)) && first.is_none() {
                        first = Some(i);
                    }
                    seen[i] = st.len();
                }
            }
        }
    }
    let r = m.result();
    let fold = |t: &mut CapStats| {
        t.fights += 1;
        t.capped += r.capped as u32;
        t.gap_sum += gap;
        t.gap_n += n;
        t.ticks += n;
        for i in 0..2 {
            let st = m.move_starts(i);
            t.swings[i] += st.iter().filter(|&&id| is_attack(id)).count() as f64;
            t.parries[i] += st.iter().filter(|&&id| id == MoveId::Parry).count() as f64;
            t.guards[i] += st.iter().filter(|&&id| id == MoveId::Guard).count() as f64;
            t.idles[i] += st.iter().filter(|&&id| id == MoveId::Idle).count() as f64;
        }
        match first {
            Some(i) => t.first[i] += 1,
            None => t.no_attack += 1,
        }
    };
    fold(all);
    if r.capped {
        fold(s);
    }
}

fn load(dir: &Path, role: &str) -> Result<Vec<Genome>, String> {
    let mut v: Vec<(u32, Genome)> = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let name = e.map_err(|e| e.to_string())?.file_name().to_string_lossy().to_string();
        if let Some(g) = name.strip_prefix(&format!("champion_{}_gen_", role)).and_then(|r| r.strip_suffix(".json")).and_then(|r| r.parse::<u32>().ok()) {
            v.push((g, Genome::from_json(&std::fs::read_to_string(dir.join(&name)).map_err(|e| e.to_string())?)?));
        }
    }
    v.sort_by_key(|x| x.0);
    Ok(v.into_iter().map(|x| x.1).collect())
}

fn line(label: &str, s: &CapStats) -> String {
    let n = s.fights.max(1) as f64;
    let t = s.ticks.max(1.0);
    format!(
        "| {} | {} | {:.2} | {:.2}/{:.2} | {:.1}/{:.1} | {:.1}/{:.1} | {:.1}/{:.1} | {}/{}/{} |\n",
        label, s.fights, s.gap_sum / s.gap_n.max(1.0), s.swings[0] / t * 100.0, s.swings[1] / t * 100.0,
        s.parries[0] / n, s.parries[1] / n, s.guards[0] / n, s.guards[1] / n, s.idles[0] / n, s.idles[1] / n,
        s.first[0], s.first[1], s.no_attack
    )
}

/// Final champions vs earlier champions (the eval setting) for each run; table of capped-fight behaviour.
pub fn capdiag(runs: &[String], seeds: u64) -> Result<String, String> {
    let mut out = String::from("| run | fights | mean gap m | swings/100 ticks h/m | parry starts/fight h/m | guard starts/fight h/m | idle starts/fight h/m | first attacker h/m/none |\n|---|---|---|---|---|---|---|---|\n");
    for r in runs {
        let dir = Path::new(r);
        let (h, m) = (load(dir, "hero")?, load(dir, "monster")?);
        let (fh, fm) = (h.last().unwrap().clone(), m.last().unwrap().clone());
        let (mut cap, mut all) = (CapStats::default(), CapStats::default());
        for sd in 0..seeds {
            for e in &m[..m.len().saturating_sub(1)] {
                play(Brain::Net(fh.clone()), Brain::Net(e.clone()), 1000 + sd, &mut cap, &mut all);
            }
            for e in &h[..h.len().saturating_sub(1)] {
                play(Brain::Net(e.clone()), Brain::Net(fm.clone()), 1000 + sd, &mut cap, &mut all);
            }
        }
        let name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        out.push_str(&line(&format!("{} all", name), &all));
        out.push_str(&line(&format!("{} capped ({:.3})", name, cap.fights as f64 / all.fights.max(1) as f64), &cap));
    }
    Ok(out)
}
