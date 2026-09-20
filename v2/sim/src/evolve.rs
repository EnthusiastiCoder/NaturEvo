//! Co-evolution of hero and monster populations. Deterministic for a given seed
//! regardless of thread count: every match is a pure function of its inputs and
//! results are collected by index.
use crate::config::ArenaConfig;
use crate::fighter::Role;
use crate::genome::Genome;
use crate::attacks::{Rank, Species};
use crate::fighter::CombatState;
use crate::log::Event;
use crate::matchup::{play, Match, MatchResult, Outcome, Setup};
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

const HOF_CAP: usize = 256;
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

/// Species and rank of the monster in a training match, by seed: species `seed % 4`
/// (Minion, Rogue, Warrior, Mage), every 8th seed Elite, every 32nd Boss (Warrior or Mage).
pub fn species_for(seed: u64) -> (Species, Rank) {
    if seed % 32 == 0 {
        let sp = if (seed / 32) % 2 == 0 { Species::Warrior } else { Species::Mage };
        return (sp, Rank::Boss);
    }
    let sp = [Species::Minion, Species::Rogue, Species::Warrior, Species::Mage][(seed % 4) as usize];
    (sp, if seed % 8 == 0 { Rank::Elite } else { Rank::Grass })
}

/// Reward and penalty terms of one match for one role (before weighting by the config).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Terms {
    pub punish_hits: u32,
    pub statuses: u32,
    pub punish_reward: f32,
    pub status_reward: f32,
    pub unseen_guard_share: f32,
    pub passivity_share: f32,
}

fn is_guard(s: CombatState) -> bool {
    matches!(s, CombatState::GuardRaise | CombatState::Blocking | CombatState::GuardRecover)
}

pub fn terms(role: Role, res: &MatchResult, cfg: &ArenaConfig) -> Terms {
    let ri = role.index();
    let ft = &cfg.fitness;
    let n = res.log.frames.len().max(1) as f32;
    let (mut punish_hits, mut mask) = (0u32, 0u8);
    let (mut unseen, mut passive) = (0u32, 0u32);
    let mut since = 0u32;
    for f in &res.log.frames {
        let mut damaged = false;
        for e in &f.ev {
            match e {
                Event::Hit { by, dmg, punish, fx, .. } => {
                    if *dmg > 0.0 {
                        damaged = true;
                    }
                    if *by == ri {
                        if *punish {
                            punish_hits += 1;
                        }
                        for k in fx {
                            mask |= 1 << (*k as u8 & 7);
                        }
                    }
                }
                Event::Dot { dmg, .. } if *dmg > 0.0 => damaged = true,
                _ => {}
            }
        }
        if damaged {
            since = 0;
        } else {
            since += 1;
        }
        if since > ft.passivity_ticks {
            passive += 1;
        }
        let d = ((f.pos[0].0 - f.pos[1].0).powi(2) + (f.pos[0].1 - f.pos[1].1).powi(2)).sqrt();
        if is_guard(f.state[ri]) && (!f.seen[ri] || d > ft.unseen_guard_dist) {
            unseen += 1;
        }
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
    // Anti-stalemate: a timeout draw costs, and so does lingering without winning.
    if res.outcome == Outcome::Draw {
        fitness -= DRAW_PENALTY;
    }
    if !win {
        fitness -= 1.0 * res.ticks as f32 / cfg.max_ticks as f32;
    }
    // Anti-spam: dodging most of the time is a way to avoid combat; reward a mix with blocking.
    let (fa, fb, fd) = (c[0] / n, c[1] / n, c[2] / n);
    fitness -= 3.0 * (fd - 0.25).max(0.0);
    fitness += 1.5 * fb.min(0.12) + 0.5 * fa.min(0.3);
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
    w[1 * N_IN + crate::sense::idx::ENEMY_SIN] = 4.0;
    w[2 * N_IN + crate::sense::idx::IN_RANGE] = 4.0;
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

/// Held-out champion panel: fixed mix of fresh random opponents and evenly spaced past champions.
const PANEL_RANDOM: usize = 6;
const PANEL_PAST: usize = 12;
/// Copies of the rival champion in the panel (hero panel / monster panel).
const PANEL_CURRENT_HERO: usize = 8;
const PANEL_CURRENT_MONSTER: usize = 2;
/// Weight of the worst group in the blended panel score (tenths).
const GROUP_MIN_WEIGHT: f32 = 0.5;

pub struct Trainer {
    cfg: TrainConfig,
    arena_cfg: ArenaConfig,
    pops: [Vec<Genome>; 2],
    hof: [Vec<Genome>; 2],
    champs: [Genome; 2],
    stats: Vec<GenStats>,
}

impl Trainer {
    pub fn new(cfg: TrainConfig) -> Trainer {
        let mut r = Rng::new(cfg.seed ^ 0x1217_1717);
        let pop = cfg.pop.max(2);
        let hero: Vec<Genome> = (0..pop).map(|_| Genome::random(Role::Hero, &mut r)).collect();
        let mon: Vec<Genome> = (0..pop).map(|_| Genome::random(Role::Monster, &mut r)).collect();
        let champs = [hero[0].clone(), mon[0].clone()];
        Trainer { cfg, arena_cfg: ArenaConfig::default(), pops: [hero, mon], hof: [Vec::new(), Vec::new()], champs, stats: Vec::new() }
    }

    pub fn config(&self) -> &TrainConfig {
        &self.cfg
    }
    pub fn stats(&self) -> &[GenStats] {
        &self.stats
    }
    pub fn hero_champion(&self) -> &Genome {
        &self.champs[0]
    }
    pub fn monster_champion(&self) -> &Genome {
        &self.champs[1]
    }
    pub fn hall_of_fame(&self, role: Role) -> &[Genome] {
        &self.hof[role.index()]
    }

    /// Evaluate both populations, record stats and champions, then breed the next generation.
    pub fn run_generation(&mut self) {
        let gen = self.stats.len() as u32;
        let cfg = self.arena_cfg.clone();
        let mut rng = gen_rng(self.cfg.seed, gen, 0xE1);
        let mut best = [0usize; 2];
        let mut fits: [Vec<f32>; 2] = [Vec::new(), Vec::new()];
        let mut mean = [0.0f32; 2];
        let mut freq = [ActionFreq::default(); 2];
        let (mut tick_sum, mut tick_n) = (0.0f64, 0usize);

        for ri in 0..2 {
            let role = if ri == 0 { Role::Hero } else { Role::Monster };
            let oi = 1 - ri;
            let pop = self.pops[ri].len();
            // Opponent plan per individual: (opponent genome index kind, seed).
            let mut plan: Vec<Vec<(Genome, u64)>> = Vec::with_capacity(pop);
            {
                // Common random numbers: every individual meets the same opponents and arenas this generation.
                let mut v = Vec::new();
                for _ in 0..4 {
                    let j = (rng.next_u64() % self.pops[oi].len() as u64) as usize;
                    v.push((self.pops[oi][j].clone(), rng.next_u64()));
                }
                // One fresh random opponent keeps a broad baseline in the mix.
                for _ in 0..2 {
                    v.push((Genome::random(if ri == 0 { Role::Monster } else { Role::Hero }, &mut rng), rng.next_u64()));
                }
                if !self.hof[oi].is_empty() {
                    for _ in 0..self.cfg.hof_sample {
                        let j = (rng.next_u64() % self.hof[oi].len() as u64) as usize;
                        v.push((self.hof[oi][j].clone(), rng.next_u64()));
                    }
                }
                for _ in 0..pop {
                    plan.push(v.clone());
                }
            }
            let pop_ref = &self.pops[ri];
            let plan_ref = &plan;
            let cfg_ref = &cfg;
            let res: Vec<Vec<MatchScore>> = par_map(pop, |i| {
                plan_ref[i]
                    .iter()
                    .enumerate()
                    .map(|(_, (opp, s))| score_match(role, &pop_ref[i], opp, *s, cfg_ref, None))
                    .collect()
            });
            let mut f = Vec::with_capacity(pop);
            let (mut a, mut b, mut c, mut n) = (0.0f64, 0.0f64, 0.0f64, 0usize);
            for r in &res {
                let avg = r.iter().map(|x| x.fitness).sum::<f32>() / r.len().max(1) as f32;
                let low = r.iter().map(|x| x.fitness).fold(f32::INFINITY, f32::min);
                let mx = 0.5;
                let m = if low.is_finite() { mx * avg + (1.0 - mx) * low } else { 0.0 };
                f.push(m);
                for x in r {
                    a += x.freq[0] as f64;
                    b += x.freq[1] as f64;
                    c += x.freq[2] as f64;
                    tick_sum += x.ticks as f64;
                    tick_n += 1;
                }
                n += r.len();
            }
            let nn = n.max(1) as f64;
            freq[ri] = ActionFreq { attack: (a / nn) as f32, block: (b / nn) as f32, dodge: (c / nn) as f32 };
            let mut bi = 0;
            for (i, v) in f.iter().enumerate() {
                if *v > f[bi] {
                    bi = i;
                }
            }
            best[ri] = bi;
            mean[ri] = (f.iter().map(|x| *x as f64).sum::<f64>() / f.len() as f64) as f32;
            if mean[ri] > f[bi] {
                mean[ri] = f[bi];
            }
            fits[ri] = f;
        }

        // Champions and win rates (champion vs HoF sample, and vs a fixed random baseline).
        let pop_best = [
            fits[0].iter().cloned().fold(f32::MIN, f32::max),
            fits[1].iter().cloned().fold(f32::MIN, f32::max),
        ];
        let mut champs = self.champs.clone();
        // Monster champion is picked first; the hero is then judged against the fresh monster champion.
        for ri in [1usize, 0] {
            // Re-rank the top few on fresh opponents so the champion is not just a lucky draw.
            let role = if ri == 0 { Role::Hero } else { Role::Monster };
            let oi = 1 - ri;
            let mut order: Vec<usize> = (0..fits[ri].len()).collect();
            order.sort_by(|a, b| fits[ri][*b].partial_cmp(&fits[ri][*a]).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(b)));
            let mut cand_g: Vec<Genome> = order.iter().take(6).map(|&i| self.pops[ri][i].clone()).collect();
            let n_pop_c = cand_g.len();
            if gen > 0 {
                cand_g.push(self.champs[ri].clone());
            }
            // Panel: fresh randoms, the opposing current champion, and an evenly spaced slice of its whole history.
            let mut opps: Vec<(Genome, u64)> = (0..PANEL_RANDOM).map(|_| (Genome::random(role.other(), &mut rng), rng.next_u64())).collect();
            let hl = self.hof[oi].len();
            if hl > 0 {
                for _ in 0..if ri == 0 { PANEL_CURRENT_HERO } else { PANEL_CURRENT_MONSTER } {
                    let cur = if ri == 1 { self.hof[oi][hl / 2].clone() } else { champs[oi].clone() };
                    opps.push((cur, rng.next_u64()));
                }
                let k = hl.min(PANEL_PAST);
                for q in 0..k {
                    opps.push((self.hof[oi][q * hl / k].clone(), rng.next_u64()));
                }
            }
            let (cfg_ref, opps_ref, cands_ref) = (&cfg, &opps, &cand_g);
            // Group of each panel opponent: 0 fresh random, 1 the opposing champion, 2 past champions.
            let n_cur = if hl > 0 { if ri == 0 { PANEL_CURRENT_HERO } else { PANEL_CURRENT_MONSTER } } else { 0 };
            let n_rand = PANEL_RANDOM;
            let group = |i: usize| if i < n_rand { 0 } else if i < n_rand + n_cur { 1 } else { 2 };
            let per: Vec<[f32; 3]> = par_map(cand_g.len(), |c| {
                let mut tot = [0.0f32; 3];
                let mut cnt = [0.0f32; 3];
                for (i, (o, s)) in opps_ref.iter().enumerate() {
                    for sd in [*s, s ^ 0x5EED_5EED, s ^ 0x7A11_7A11, s ^ 0x1357_9BDF, s ^ 0x2468_ACE0] {
                        let m = score_match(role, &cands_ref[c], o, sd, cfg_ref, None);
                        tot[group(i)] += m.fitness + if m.win { 3.0 } else { 0.0 };
                        cnt[group(i)] += 1.0;
                    }
                }
                [tot[0] / cnt[0].max(1.0), tot[1] / cnt[1].max(1.0), tot[2] / cnt[2].max(1.0)]
            });
            // Blend of the average and the worst group: the champion must hold up against random
            // opponents, the current rival and its own past, not trade one for another.
            let n_fights = (opps.len() * 5) as f32;
            let sc: Vec<f32> = per
                .iter()
                .map(|g| {
                    let used: Vec<f32> = (0..3).filter(|k| if *k == 0 { n_rand > 0 } else if *k == 1 { n_cur > 0 } else { hl > 0 }).map(|k| g[k]).collect();
                    let mean = used.iter().sum::<f32>() / used.len() as f32;
                    let min = used.iter().cloned().fold(f32::INFINITY, f32::min);
                    let w = GROUP_MIN_WEIGHT;
                    n_fights * ((1.0 - w) * mean + w * min)
                })
                .collect();
            // Hysteresis: a challenger must clearly beat the sitting champion.
            let mut sc = sc;
            if gen > 0 {
                let last = sc.len() - 1;
                sc[last] += 0.08 * sc[last].abs() + 2.0;
            }
            let mut bi = 0;
            for (i, v) in sc.iter().enumerate() {
                if *v > sc[bi] {
                    bi = i;
                }
            }
            // A kept previous champion competes for the title but is not a population member.
            let kept_prev = bi >= n_pop_c;
            best[ri] = if kept_prev { order[0] } else { order[bi] };
            let mut c = cand_g[bi].clone();
            c.generation = if kept_prev { c.generation } else { gen };
            c.fitness = if kept_prev { c.fitness } else { fits[ri][best[ri]] };
            champs[ri] = c;
        }
        let mut wr_hof = [0.0f32; 2];
        let mut wr_rnd = [0.0f32; 2];
        for ri in 0..2 {
            let role = if ri == 0 { Role::Hero } else { Role::Monster };
            let oi = 1 - ri;
            let opp_role = role.other();
            let mut opps: Vec<(Genome, u64)> = Vec::new();
            if self.hof[oi].is_empty() {
                let mut rr = Rng::new(self.cfg.seed ^ 0xF00D);
                for _ in 0..BASELINE_N {
                    opps.push((Genome::random(opp_role, &mut rr), rr.next_u64()));
                }
            } else {
                for _ in 0..BASELINE_N {
                    let j = (rng.next_u64() % self.hof[oi].len() as u64) as usize;
                    opps.push((self.hof[oi][j].clone(), rng.next_u64()));
                }
            }
            let champ = &champs[ri];
            let cfg_ref = &cfg;
            let w: Vec<bool> = par_map(opps.len(), |i| score_match(role, champ, &opps[i].0, opps[i].1, cfg_ref, None).win);
            wr_hof[ri] = w.iter().filter(|x| **x).count() as f32 / w.len() as f32;
            // Fixed baseline: identical random opponents every generation.
            let mut rr = Rng::new(0xBA5E ^ ri as u64);
            let base: Vec<(Genome, u64)> = (0..BASELINE_N).map(|_| (Genome::random(opp_role, &mut rr), rr.next_u64())).collect();
            let w: Vec<bool> = par_map(base.len(), |i| score_match(role, champ, &base[i].0, base[i].1, cfg_ref, None).win);
            wr_rnd[ri] = w.iter().filter(|x| **x).count() as f32 / w.len() as f32;
        }

        self.stats.push(GenStats {
            gen,
            best_fitness: [pop_best[0], pop_best[1]],
            mean_fitness: mean,
            win_rate_vs_hof: wr_hof,
            win_rate_vs_random: wr_rnd,
            action_freq: freq,
            avg_ticks: (tick_sum / tick_n.max(1) as f64) as f32,
        });
        for ri in 0..2 {
            self.hof[ri].push(champs[ri].clone());
            if self.hof[ri].len() > HOF_CAP {
                self.hof[ri].remove(0);
            }
        }
        self.champs = champs;

        // Breed.
        let mut br = gen_rng(self.cfg.seed, gen, 0xB7);
        for ri in 0..2 {
            let fit = &fits[ri];
            let mut order: Vec<usize> = (0..fit.len()).collect();
            order.sort_by(|a, b| fit[*b].partial_cmp(&fit[*a]).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(b)));
            let old = &self.pops[ri];
            let mut next: Vec<Genome> = Vec::with_capacity(old.len());
            for &i in order.iter().take(ELITES.min(old.len())) {
                let mut g = old[i].clone();
                g.generation = gen + 1;
                next.push(g);
            }
            while next.len() < old.len() {
                let a = &old[tournament(fit, &mut br)];
                let b = &old[tournament(fit, &mut br)];
                let mut child = if br.f32() < 0.5 { Genome::crossover(a, b, &mut br) } else { a.clone() };
                child.mutate(&mut br, 0.05, 0.25 * (1.0 - gen as f32 / 150.0).max(0.3));
                child.generation = gen + 1;
                child.fitness = 0.0;
                next.push(child);
            }
            if gen == 0 {
                // Seeded immigrants: after the random generation, half the new population is primed.
                for (k, g) in next.iter_mut().enumerate().skip(ELITES) {
                    if k % 2 == 0 {
                        prime(g, 2);
                    }
                }
            }
            self.pops[ri] = next;
        }
    }

    pub fn checkpoint(&self) -> String {
        let c = &self.cfg;
        let mut s = String::from("arena-checkpoint-v1\n");
        let _ = writeln!(s, "cfg {} {} {} {} {}", c.seed, c.generations, c.pop, c.k_opponents, c.hof_sample);
        let _ = writeln!(s, "stats {}", self.stats.len());
        for st in &self.stats {
            let _ = writeln!(s, "{}", st.to_nums());
        }
        for g in &self.champs {
            let _ = writeln!(s, "champ {}", g.to_json());
        }
        for (name, sets) in [("pop", &self.pops), ("hof", &self.hof)] {
            for ri in 0..2 {
                let _ = writeln!(s, "{} {} {}", name, ri, sets[ri].len());
                for g in &sets[ri] {
                    let _ = writeln!(s, "{}", g.to_json());
                }
            }
        }
        s
    }

    pub fn resume(text: &str) -> Result<Trainer, String> {
        let mut it = text.lines();
        let mut next = || it.next().ok_or_else(|| "truncated checkpoint".to_string());
        if next()? != "arena-checkpoint-v1" {
            return Err("not a checkpoint".into());
        }
        let line = next()?;
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 6 || p[0] != "cfg" {
            return Err("bad cfg line".into());
        }
        let bad = |_| "bad cfg value".to_string();
        let cfg = TrainConfig {
            seed: p[1].parse().map_err(bad)?,
            generations: p[2].parse().map_err(bad)?,
            pop: p[3].parse().map_err(bad)?,
            k_opponents: p[4].parse().map_err(bad)?,
            hof_sample: p[5].parse().map_err(bad)?,
        };
        let line = next()?;
        let n: usize = line.strip_prefix("stats ").ok_or("bad stats header")?.trim().parse().map_err(|_| "bad stats count")?;
        let mut stats = Vec::new();
        for _ in 0..n {
            stats.push(GenStats::from_nums(next()?)?);
        }
        let mut champs = Vec::new();
        for _ in 0..2 {
            let l = next()?;
            champs.push(Genome::from_json(l.strip_prefix("champ ").ok_or("bad champ line")?)?);
        }
        let mut sets: Vec<[Vec<Genome>; 2]> = Vec::new();
        for name in ["pop", "hof"] {
            let mut both: [Vec<Genome>; 2] = [Vec::new(), Vec::new()];
            for ri in 0..2 {
                let l = next()?;
                let pre = format!("{} {} ", name, ri);
                let cnt: usize = l.strip_prefix(&pre).ok_or("bad set header")?.trim().parse().map_err(|_| "bad set count")?;
                for _ in 0..cnt {
                    both[ri].push(Genome::from_json(next()?)?);
                }
            }
            sets.push(both);
        }
        let hof = sets.pop().unwrap();
        let pops = sets.pop().unwrap();
        if pops[0].is_empty() || pops[1].is_empty() {
            return Err("empty population".into());
        }
        let c1 = champs.pop().unwrap();
        let c0 = champs.pop().unwrap();
        Ok(Trainer { cfg, arena_cfg: ArenaConfig::default(), pops, hof, champs: [c0, c1], stats })
    }
}

fn num<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, String> {
    match args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)) {
        Some(v) => v.parse().map_err(|_| format!("bad {}", name)),
        None => Ok(default),
    }
}

/// `train --seed N --gens N --out DIR [--pop N] [--k N] [--hof N] [--resume FILE]`
pub fn train_cli(args: &[String]) -> Result<(), String> {
    use std::io::Write as _;
    let out = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).ok_or("missing --out")?;
    let gens: u32 = num(args, "--gens", 200)?;
    let cfg = TrainConfig {
        seed: num(args, "--seed", 1)?,
        generations: gens,
        pop: num(args, "--pop", 64)?,
        k_opponents: num(args, "--k", 4)?,
        hof_sample: num(args, "--hof", 8)?,
    };
    let dir = std::path::Path::new(out);
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {}", out, e))?;
    let io = |e: std::io::Error| e.to_string();
    let mut t = match args.iter().position(|a| a == "--resume").and_then(|i| args.get(i + 1)) {
        Some(p) => Trainer::resume(&std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p, e))?)?,
        None => Trainer::new(cfg),
    };
    let done = t.stats().len();
    let mut jsonl = std::fs::OpenOptions::new().create(true).write(true).truncate(done == 0).append(done > 0).open(dir.join("training.jsonl")).map_err(io)?;
    let csv_path = dir.join("training.csv");
    let mut csv = std::fs::OpenOptions::new().create(true).write(true).truncate(done == 0).append(done > 0).open(&csv_path).map_err(io)?;
    if done == 0 {
        writeln!(csv, "{}", GenStats::csv_header()).map_err(io)?;
    }
    let total = t.config().generations as usize;
    let save = |t: &Trainer, g: u32| -> Result<(), String> {
        let (h, m) = (t.hero_champion(), t.monster_champion());
        let w = |name: String, body: String| std::fs::write(dir.join(&name), body).map_err(|e| format!("{}: {}", name, e));
        w(format!("champion_hero_gen_{}.json", g), h.to_json())?;
        w(format!("champion_monster_gen_{}.json", g), m.to_json())?;
        let cfg = ArenaConfig::default();
        let res = play(Arena::generate(g as u64, &cfg), h.clone(), m.clone(), g as u64, &cfg);
        w(format!("fight_gen_{}.json", g), res.log.to_json())?;
        w("checkpoint.txt".to_string(), t.checkpoint())
    };
    for g in done..total {
        t.run_generation();
        let s = t.stats().last().unwrap();
        writeln!(jsonl, "{}", s.to_json()).map_err(io)?;
        writeln!(csv, "{}", s.to_csv()).map_err(io)?;
        eprintln!(
            "gen {} fit h{:.2}/m{:.2} wr_rnd h{:.2}/m{:.2} ticks {:.0}",
            s.gen, s.best_fitness[0], s.best_fitness[1], s.win_rate_vs_random[0], s.win_rate_vs_random[1], s.avg_ticks
        );
        if (g + 1) % 10 == 0 || g + 1 == total {
            save(&t, g as u32)?;
        }
    }
    Ok(())
}

