//! Co-evolution of hero and monster populations. Deterministic for a given seed
//! regardless of thread count: every match is a pure function of its inputs and
//! results are collected by index.
use crate::config::ArenaConfig;
use crate::fighter::Role;
use crate::genome::Genome;
use crate::matchup::{play, Match, MatchResult, Outcome};
use crate::rng::Rng;
use crate::world::Arena;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub struct TrainConfig {
    pub seed: u64,
    pub generations: u32,
    pub pop: usize,
    pub k_opponents: usize,
    pub hof_sample: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActionFreq {
    pub attack: f32,
    pub block: f32,
    pub dodge: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenStats {
    pub gen: u32,
    pub best_fitness: [f32; 2],
    pub mean_fitness: [f32; 2],
    pub win_rate_vs_hof: [f32; 2],
    pub win_rate_vs_random: [f32; 2],
    pub action_freq: [ActionFreq; 2],
    pub avg_ticks: f32,
}

impl GenStats {
    pub fn to_json(&self) -> String {
        let r = ["hero", "monster"];
        let mut s = format!("{{\"gen\":{}", self.gen);
        let _ = write!(s, ",\"avg_ticks\":{}", self.avg_ticks);
        for (key, v) in [
            ("best_fitness", &self.best_fitness),
            ("mean_fitness", &self.mean_fitness),
            ("win_rate_vs_hof", &self.win_rate_vs_hof),
            ("win_rate_vs_random", &self.win_rate_vs_random),
        ] {
            let _ = write!(s, ",\"{}\":{{\"hero\":{},\"monster\":{}}}", key, v[0], v[1]);
        }
        let _ = write!(s, ",\"action_freq\":{{");
        for i in 0..2 {
            let a = &self.action_freq[i];
            if i > 0 {
                s.push(',');
            }
            let _ = write!(s, "\"{}\":{{\"attack\":{},\"block\":{},\"dodge\":{}}}", r[i], a.attack, a.block, a.dodge);
        }
        s.push_str("}}");
        s
    }

    pub fn csv_header() -> &'static str {
        "gen,avg_ticks,best_hero,best_monster,mean_hero,mean_monster,hof_wr_hero,hof_wr_monster,rnd_wr_hero,rnd_wr_monster,hero_attack,hero_block,hero_dodge,monster_attack,monster_block,monster_dodge"
    }

    pub fn to_csv(&self) -> String {
        let a = &self.action_freq;
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.gen, self.avg_ticks, self.best_fitness[0], self.best_fitness[1], self.mean_fitness[0], self.mean_fitness[1],
            self.win_rate_vs_hof[0], self.win_rate_vs_hof[1], self.win_rate_vs_random[0], self.win_rate_vs_random[1],
            a[0].attack, a[0].block, a[0].dodge, a[1].attack, a[1].block, a[1].dodge
        )
    }

    fn to_nums(&self) -> String {
        let a = &self.action_freq;
        let v: Vec<String> = [
            self.gen as f32, self.avg_ticks, self.best_fitness[0], self.best_fitness[1], self.mean_fitness[0], self.mean_fitness[1],
            self.win_rate_vs_hof[0], self.win_rate_vs_hof[1], self.win_rate_vs_random[0], self.win_rate_vs_random[1],
            a[0].attack, a[0].block, a[0].dodge, a[1].attack, a[1].block, a[1].dodge,
        ]
        .iter()
        .map(|x| format!("{}", x))
        .collect();
        v.join(" ")
    }

    fn from_nums(s: &str) -> Result<GenStats, String> {
        let v: Vec<f32> = s.split_whitespace().map(|x| x.parse::<f32>().map_err(|e| e.to_string())).collect::<Result<_, _>>()?;
        if v.len() != 16 {
            return Err("bad stats line".into());
        }
        let af = |i: usize| ActionFreq { attack: v[i], block: v[i + 1], dodge: v[i + 2] };
        Ok(GenStats {
            gen: v[0] as u32,
            avg_ticks: v[1],
            best_fitness: [v[2], v[3]],
            mean_fitness: [v[4], v[5]],
            win_rate_vs_hof: [v[6], v[7]],
            win_rate_vs_random: [v[8], v[9]],
            action_freq: [af(10), af(13)],
        })
    }
}

const HOF_CAP: usize = 64;
const ELITES: usize = 2;
const BASELINE_N: usize = 10;

/// Per-match outcome for the evaluated role.
#[derive(Clone, Copy, Debug)]
struct MatchScore {
    fitness: f32,
    win: bool,
    ticks: u32,
    freq: [f32; 3],
}

fn par_map<T: Send, F: Fn(usize) -> T + Sync>(n: usize, f: F) -> Vec<T> {
    let threads = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(1).min(n.max(1));
    if threads <= 1 {
        return (0..n).map(&f).collect();
    }
    let mut out: Vec<Option<T>> = (0..n).map(|_| None).collect();
    let chunk = (n + threads - 1) / threads;
    std::thread::scope(|sc| {
        let f = &f;
        for (ci, slot) in out.chunks_mut(chunk).enumerate() {
            sc.spawn(move || {
                for (k, s) in slot.iter_mut().enumerate() {
                    *s = Some(f(ci * chunk + k));
                }
            });
        }
    });
    out.into_iter().map(|x| x.unwrap()).collect()
}

fn score_match(role: Role, me: &Genome, opp: &Genome, seed: u64, cfg: &ArenaConfig, near: Option<f32>) -> MatchScore {
    let (h, m) = if role == Role::Hero { (me.clone(), opp.clone()) } else { (opp.clone(), me.clone()) };
    // `near` starts the duel at close range (curriculum: combat skills get learned before navigation).
    let mut mt = Match::new(Arena::generate(seed, cfg), h, m, seed, cfg);
    if let Some(d) = near {
        mt.place_for_duel(d);
    }
    while !mt.finished() {
        mt.step();
    }
    let res = MatchResult {
        outcome: mt.outcome(),
        ticks: mt.tick(),
        hero_hp: mt.hp(Role::Hero),
        monster_hp: mt.hp(Role::Monster),
        damage_dealt: mt.damage_dealt(),
        log: mt.log().clone(),
    };
    let arena = mt.arena().clone();
    let ri = role.index();
    let dealt = res.damage_dealt[ri] / cfg.max_hp;
    let taken = res.damage_dealt[1 - ri] / cfg.max_hp;
    let win = matches!((res.outcome, role), (Outcome::HeroWin, Role::Hero) | (Outcome::MonsterWin, Role::Monster));
    let mut fitness = 4.0 * dealt - 1.0 * taken;
    if win {
        fitness += 1.0 + 0.5 * (1.0 - res.ticks as f32 / cfg.max_ticks as f32);
        let own = if role == Role::Hero { res.hero_hp } else { res.monster_hp };
        fitness += 5.0 * own / cfg.max_hp;
    }
    let n = res.log.frames.len().max(1) as f32;
    let mut c = [0.0f32; 3];
    let mut shape = 0.0f32;
    let (aw, ah) = arena.size();
    let mut field: Vec<u16> = Vec::new();
    for (fi, f) in res.log.frames.iter().enumerate() {
        let (me, op) = (f.pos[ri], f.pos[1 - ri]);
        let (dx, dy) = (op.0 - me.0, op.1 - me.1);
        if fi % 10 == 0 {
            field = path_field(&arena, aw, ah, op);
        }
        let (tx, ty) = ((me.0.floor().max(0.0) as usize).min(aw - 1), (me.1.floor().max(0.0) as usize).min(ah - 1));
        let pd = field[ty * aw + tx];
        let eu = (dx * dx + dy * dy).sqrt();
        // Walking distance gives a gradient around obstacles; fall back to straight-line if unreachable.
        let dist = if pd == u16::MAX { eu * 1.5 } else { (pd as f32).max(eu) };
        let close = 0.5 / (1.0 + dist / 3.0) + 0.5 * (1.0 - dist / 40.0).max(0.0);
        let align = (dy.atan2(dx) - f.facing[ri]).cos().max(0.0);
        shape += close * (0.5 + 0.5 * align);
        if f.action[ri].attack {
            shape += 1.5 * close * close * align;
        }
        let a = &f.action[ri];
        c[0] += a.attack as u8 as f32;
        c[1] += a.block as u8 as f32;
        c[2] += a.dodge as u8 as f32;
    }
    fitness += 0.5 * shape / n;
    MatchScore { fitness, win, ticks: res.ticks, freq: [c[0] / n, c[1] / n, c[2] / n] }
}

fn path_field(arena: &Arena, w: usize, h: usize, from: (f32, f32)) -> Vec<u16> {
    let mut d = vec![u16::MAX; w * h];
    let sx = (from.0.floor().max(0.0) as usize).min(w - 1);
    let sy = (from.1.floor().max(0.0) as usize).min(h - 1);
    let tiles = arena.tiles();
    let mut q = std::collections::VecDeque::new();
    d[sy * w + sx] = 0;
    q.push_back((sx, sy));
    while let Some((x, y)) = q.pop_front() {
        let cur = d[y * w + x];
        let mut try_push = |nx: i64, ny: i64| {
            if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                let i = ny as usize * w + nx as usize;
                if !tiles[i] && d[i] == u16::MAX {
                    d[i] = cur + 1;
                    q.push_back((nx as usize, ny as usize));
                }
            }
        };
        try_push(x as i64 + 1, y as i64);
        try_push(x as i64 - 1, y as i64);
        try_push(x as i64, y as i64 + 1);
        try_push(x as i64, y as i64 - 1);
    }
    d
}

/// Warm start: walk forward, steer toward a seen enemy; level 2 also swings when in range.
fn prime(g: &mut Genome, level: u32) {
    use crate::net::{N_HID, N_OUT};
    use crate::sense::N_IN;
    let w = &mut g.weights;
    let b1 = N_HID * N_IN + N_HID * N_HID;
    let w2 = b1 + N_HID;
    let b2 = w2 + N_OUT * N_HID;
    for x in w.iter_mut() {
        *x *= 0.3;
    }
    for i in 0..N_OUT {
        w[b2 + i] = 0.0;
    }
    w[b1] = 3.0;
    w[1 * N_IN + 12] = 4.0;
    w[2 * N_IN + 38] = 4.0;
    w[w2] = 1.5;
    w[w2 + 2 * N_HID + 1] = 1.2;
    if level >= 2 {
        w[w2 + 3 * N_HID] = -1.0;
        w[w2 + 3 * N_HID + 2] = 2.5;
    }
}

fn gen_rng(seed: u64, gen: u32, salt: u64) -> Rng {
    let mut r = Rng::new(seed ^ salt);
    let base = r.next_u64();
    Rng::new(base.wrapping_add((gen as u64 + 1).wrapping_mul(0xD1B5_4A32_D192_ED03)))
}

fn tournament(fit: &[f32], r: &mut Rng) -> usize {
    let mut best = (r.next_u64() % fit.len() as u64) as usize;
    for _ in 0..2 {
        let c = (r.next_u64() % fit.len() as u64) as usize;
        if fit[c] > fit[best] {
            best = c;
        }
    }
    best
}

pub struct Trainer {
    cfg: TrainConfig,
    arena_cfg: ArenaConfig,
    pops: [Vec<Genome>; 2],
    hof: [Vec<Genome>; 2],
    champs: [Genome; 2],
    stats: Vec<GenStats>,
}
