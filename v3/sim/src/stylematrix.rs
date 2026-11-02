//! Style evaluation: pair fighters (teachers or trained champions), run fights over fixed seeds and report win
//! rates and behavioural metrics per style. Measurement only; nothing here feeds back into fitness.
use crate::arena::Arena;
use crate::duel::{Brain, Match, MatchResult, Outcome};
use crate::grace::match_grace;
use crate::moveset::N_MOVES;
use crate::net::N_OUT;

/// Behaviour totals of one side across fights (divide by `n` for means).
#[derive(Clone, Debug, Default)]
pub struct SideAcc {
    pub n: u32,
    pub wins: u32,
    pub draws: u32,
    pub capped: u32,
    pub ticks: f64,
    pub distinct: f64,
    pub union_moves: [bool; N_MOVES],
    pub gap_mean: f64,
    pub engaged_gap: f64,
    pub spacing_std: f64,
    pub hits_per_swing: f64,
    pub whiff: f64,
    pub punish: f64,
    pub parry_hits: f64,
    pub defences: f64,
    pub parry_defs: f64,
    pub combo_mean: f64,
    pub combo_max: f64,
    pub jerk: f64,
    pub tip_jerk: f64,
    pub swings: f64,
    pub hits: f64,
    pub usage: [f64; N_MOVES],
}

impl SideAcc {
    pub fn add(&mut self, m: &Match, r: &MatchResult, who: usize) {
        let g = match_grace(m, who);
        self.n += 1;
        let mine = if who == 0 { Outcome::WinA } else { Outcome::WinB };
        if r.outcome == mine {
            self.wins += 1;
        }
        if r.outcome == Outcome::Draw {
            self.draws += 1;
        }
        self.capped += r.capped as u32;
        self.ticks += r.ticks as f64;
        self.distinct += g.usage.iter().filter(|u| **u > 0.0).count() as f64;
        for i in 0..N_MOVES {
            if g.usage[i] > 0.0 {
                self.union_moves[i] = true;
            }
            self.usage[i] += g.usage[i] as f64;
        }
        self.gap_mean += r.gap_mean as f64;
        self.engaged_gap += r.engaged_gap as f64;
        self.spacing_std += g.spacing_std as f64;
        self.hits_per_swing += g.hits_per_swing as f64;
        self.whiff += g.whiff_rate as f64;
        self.punish += g.punish_rate as f64;
        let d = &r.defence[who];
        let tot = (d.parry + d.block + d.dodge) as f64;
        self.defences += tot;
        self.parry_defs += d.parry as f64;
        self.combo_mean += r.combo_mean[who] as f64;
        self.combo_max += r.combo_max[who] as f64;
        self.jerk += r.jerk[who] as f64;
        self.tip_jerk += g.jerk as f64;
        self.swings += r.swings[who] as f64;
        self.hits += r.hits[who] as f64;
    }
    fn mean(&self, v: f64) -> f64 {
        v / self.n.max(1) as f64
    }
    pub fn win_rate(&self) -> f64 {
        self.mean(self.wins as f64)
    }
    pub fn distinct_moves(&self) -> f64 {
        self.mean(self.distinct)
    }
    pub fn gap(&self) -> f64 {
        self.mean(self.gap_mean)
    }
    pub fn hits_per_swing(&self) -> f64 {
        self.mean(self.hits_per_swing)
    }
    pub fn capped_share(&self) -> f64 {
        self.mean(self.capped as f64)
    }
    pub fn mean_ticks(&self) -> f64 {
        self.mean(self.ticks)
    }
    pub fn combo(&self) -> f64 {
        self.mean(self.combo_mean)
    }
    pub fn parry_share(&self) -> f64 {
        if self.defences > 0.0 { self.parry_defs / self.defences } else { 0.0 }
    }
    pub fn merge(&mut self, o: &SideAcc) {
        self.n += o.n;
        self.wins += o.wins;
        self.draws += o.draws;
        self.capped += o.capped;
        for i in 0..N_MOVES {
            self.union_moves[i] |= o.union_moves[i];
            self.usage[i] += o.usage[i];
        }
        for (a, b) in [
            (&mut self.ticks, o.ticks), (&mut self.distinct, o.distinct), (&mut self.gap_mean, o.gap_mean), (&mut self.engaged_gap, o.engaged_gap), (&mut self.spacing_std, o.spacing_std),
            (&mut self.hits_per_swing, o.hits_per_swing), (&mut self.whiff, o.whiff), (&mut self.punish, o.punish), (&mut self.parry_hits, o.parry_hits),
            (&mut self.defences, o.defences), (&mut self.parry_defs, o.parry_defs), (&mut self.combo_mean, o.combo_mean), (&mut self.combo_max, o.combo_max),
            (&mut self.jerk, o.jerk), (&mut self.tip_jerk, o.tip_jerk), (&mut self.swings, o.swings), (&mut self.hits, o.hits),
        ] {
            *a += b;
        }
    }
}

/// Fights `a` (slot 0) against `b` (slot 1) on seeds 1000..1000+seeds; `flat` removes the arena obstacles.
pub fn run_pair(a: &Brain, b: &Brain, seeds: u64, flat: bool) -> (SideAcc, SideAcc) {
    let (mut sa, mut sb) = (SideAcc::default(), SideAcc::default());
    for s in 0..seeds {
        let mut m = Match::new(1000 + s, a.clone(), b.clone());
        if flat {
            m.arena = Arena::flat(m.arena.width);
        }
        let r = m.run_to_end();
        sa.add(&m, &r, 0);
        sb.add(&m, &r, 1);
    }
    (sa, sb)
}

/// One named contestant: the brain it uses in slot 0 and in slot 1.
#[derive(Clone)]
pub struct Contestant {
    pub name: String,
    pub as_a: Brain,
    pub as_b: Brain,
}

pub struct Matrix {
    pub names: Vec<String>,
    /// wins[i][j]: share of fights i won against j (both slots), draws[i][j] likewise.
    pub wins: Vec<Vec<f64>>,
    pub draws: Vec<Vec<f64>>,
    pub per_style: Vec<SideAcc>,
}

/// All ordered pairs (i in slot 0 against j in slot 1), including i == j, on `seeds` arenas each.
pub fn matrix(cs: &[Contestant], seeds: u64, flat: bool) -> Matrix {
    let n = cs.len();
    let mut wins = vec![vec![0.0; n]; n];
    let mut draws = vec![vec![0.0; n]; n];
    let mut cnt = vec![vec![0.0f64; n]; n];
    let mut per = vec![SideAcc::default(); n];
    let jobs: Vec<(usize, usize)> = (0..n).flat_map(|i| (0..n).map(move |j| (i, j))).collect();
    let results: Vec<(usize, usize, SideAcc, SideAcc)> = std::thread::scope(|s| {
        let hs: Vec<_> = jobs
            .iter()
            .map(|&(i, j)| {
                let (a, b) = (cs[i].as_a.clone(), cs[j].as_b.clone());
                s.spawn(move || {
                    let (x, y) = run_pair(&a, &b, seeds, flat);
                    (i, j, x, y)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (i, j, x, y) in results {
        wins[i][j] += x.wins as f64;
        cnt[i][j] += x.n as f64;
        draws[i][j] += x.draws as f64;
        if i != j {
            wins[j][i] += y.wins as f64;
            cnt[j][i] += y.n as f64;
            draws[j][i] += y.draws as f64;
        }
        per[i].merge(&x);
        if i != j {
            per[j].merge(&y);
        }
    }
    for i in 0..n {
        for j in 0..n {
            let c = cnt[i][j].max(1.0);
            wins[i][j] /= c;
            draws[i][j] /= c;
        }
    }
    Matrix { names: cs.iter().map(|c| c.name.clone()).collect(), wins, draws, per_style: per }
}

pub fn markdown(title: &str, m: &Matrix) -> String {
    let mut s = format!("## {}\n\nWin rate of the row style against the column style (both slots, draws count as no win).\n\n| row \\ col |", title);
    for n in &m.names {
        s.push_str(&format!(" {} |", n));
    }
    s.push_str("\n|---|");
    for _ in &m.names {
        s.push_str("---|");
    }
    s.push('\n');
    for (i, n) in m.names.iter().enumerate() {
        s.push_str(&format!("| {} |", n));
        for j in 0..m.names.len() {
            s.push_str(&format!(" {:.2} (draw {:.2}) |", m.wins[i][j], m.draws[i][j]));
        }
        s.push('\n');
    }
    s.push_str("\n| metric |");
    for n in &m.names {
        s.push_str(&format!(" {} |", n));
    }
    s.push_str("\n|---|");
    for _ in &m.names {
        s.push_str("---|");
    }
    s.push('\n');
    type Row = (&'static str, fn(&SideAcc) -> f64);
    let rows: [Row; 16] = [
        ("win rate (all pairings)", |a| a.win_rate()),
        ("distinct moves per fight", |a| a.distinct_moves()),
        ("distinct moves (union)", |a| a.union_moves.iter().filter(|b| **b).count() as f64),
        ("mean spacing (m)", |a| a.gap()),
        ("engaged spacing (m, within 4 m)", |a| a.engaged_gap / a.n.max(1) as f64),
        ("spacing std (m)", |a| a.spacing_std / a.n.max(1) as f64),
        ("hits per swing", |a| a.hits_per_swing()),
        ("whiff rate", |a| a.whiff / a.n.max(1) as f64),
        ("punish rate", |a| a.punish / a.n.max(1) as f64),
        ("parry share of defences", |a| a.parry_share()),
        ("combo length (mean)", |a| a.combo()),
        ("combo length (longest)", |a| a.combo_max / a.n.max(1) as f64),
        ("jerk", |a| a.jerk / a.n.max(1) as f64),
        ("swings per fight", |a| a.swings / a.n.max(1) as f64),
        ("fight length (ticks)", |a| a.mean_ticks()),
        ("capped share", |a| a.capped_share()),
    ];
    for (name, f) in rows.iter() {
        s.push_str(&format!("| {} |", name));
        for a in &m.per_style {
            s.push_str(&format!(" {:.3} |", f(a)));
        }
        s.push('\n');
    }
    let _ = N_OUT;
