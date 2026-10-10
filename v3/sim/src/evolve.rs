//! Co-evolution of hero and monster populations with curriculum, hall of fame and threaded evaluation.
use crate::curriculum::{hof_enabled, opponents_per_individual, stage_for_gen, Stage};
use crate::duel::{Brain, Match, MatchResult, Outcome};
use crate::fitness::score_match;
use crate::genome::Genome;
use crate::metrics::csv_header;
use crate::rng::Rng;
use crate::robot::Role;
use std::path::PathBuf;

/// Last champions kept for self-play, saved every 10 generations.
pub struct HallOfFame {
    cap: usize,
    members: Vec<Genome>,
}

impl HallOfFame {
    pub fn new(cap: usize) -> HallOfFame {
        HallOfFame { cap, members: Vec::new() }
    }
    pub fn should_save(&self, gen: u32) -> bool {
        gen > 0 && gen % 10 == 0
    }
    pub fn push(&mut self, g: Genome) {
        self.members.push(g);
        if self.members.len() > self.cap {
            self.members.remove(0);
        }
    }
    pub fn len(&self) -> usize {
        self.members.len()
    }
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
    pub fn generations(&self) -> Vec<u32> {
        self.members.iter().map(|g| g.generation).collect()
    }
    pub fn pick(&self, rng: &mut Rng) -> &Genome {
        let i = (rng.next_u64() % self.members.len() as u64) as usize;
        &self.members[i]
    }
}

#[derive(Clone, Debug)]
pub struct TrainConfig {
    pub seed: u64,
    pub pop: usize,
    pub gens: u32,
    pub out: PathBuf,
    pub threads: usize,
    pub elites: usize,
    pub tournament: usize,
    pub mutation_rate: f32,
    pub sigma_start: f32,
    pub sigma_end: f32,
    pub name: String,
}

impl TrainConfig {
    pub fn new(seed: u64, pop: usize, gens: u32, out: PathBuf) -> TrainConfig {
        let name = out.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "run".into());
        TrainConfig { seed, pop, gens, out, threads: 1, elites: (pop / 16).max(1), tournament: 3, mutation_rate: 0.08, sigma_start: 0.25, sigma_end: 0.05, name }
    }
}

struct Job {
    hero: Option<usize>,
    mon: Option<usize>,
    a: Brain,
    b: Brain,
    seed: u64,
}

fn run_jobs(jobs: &[Job], threads: usize) -> Vec<MatchResult> {
    let n = jobs.len();
    let mut out: Vec<Option<MatchResult>> = (0..n).map(|_| None).collect();
    let t = threads.max(1).min(n.max(1));
    let chunk = ((n + t - 1) / t).max(1);
    std::thread::scope(|s| {
        for (js, os) in jobs.chunks(chunk).zip(out.chunks_mut(chunk)) {
            s.spawn(move || {
                for (j, o) in js.iter().zip(os.iter_mut()) {
                    *o = Some(Match::new(j.seed, j.a.clone(), j.b.clone()).run_to_end());
                }
            });
        }
    });
    out.into_iter().map(|o| o.expect("job result")).collect()
}

fn permutation(n: usize, rng: &mut Rng) -> Vec<usize> {
    let mut p: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        p.swap(i, j);
    }
    p
}

#[allow(clippy::too_many_arguments)]
fn make_jobs(gen: u32, stage: Stage, heroes: &[Genome], mons: &[Genome], hof_h: &HallOfFame, hof_m: &HallOfFame, seed: u64, rng: &mut Rng) -> Vec<Job> {
    let n = heroes.len();
    let mut jobs = Vec::new();
    let mut sd = seed ^ ((gen as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut next_seed = || {
        sd = sd.wrapping_add(0x1234_5677);
        sd
    };
    let brain_h = |i: usize| Brain::Net(heroes[i].clone());
    let brain_m = |i: usize| Brain::Net(mons[i].clone());
    let k = opponents_per_individual(stage);
    if stage == Stage::A {
        for i in 0..n {
            for _ in 0..k {
                let opp = if rng.f32() < 0.7 { Brain::Dummy } else { Brain::Scripted };
                jobs.push(Job { hero: Some(i), mon: None, a: brain_h(i), b: opp, seed: next_seed() });
            }
            for _ in 0..k {
                let opp = if rng.f32() < 0.7 { Brain::Dummy } else { Brain::Scripted };
                jobs.push(Job { hero: None, mon: Some(i), a: opp, b: brain_m(i), seed: next_seed() });
            }
        }
        return jobs;
    }
    // Slots per individual: self-play pairs, scripted attackers, hall of fame opponents.
    let (pairs, scripted, hof) = if stage == Stage::B {
        let p = if gen % 2 == 0 { 1 } else { 2 };
        (p, k - p, 0)
    } else if hof_enabled(stage) && !hof_h.is_empty() && !hof_m.is_empty() {
        (1, 1, 1)
    } else {
        (2, 1, 0)
    };
    for _ in 0..pairs {
        let perm = permutation(n, rng);
        for i in 0..n {
            jobs.push(Job { hero: Some(i), mon: Some(perm[i]), a: brain_h(i), b: brain_m(perm[i]), seed: next_seed() });
        }
    }
    for _ in 0..scripted {
        for i in 0..n {
            jobs.push(Job { hero: Some(i), mon: None, a: brain_h(i), b: Brain::Scripted, seed: next_seed() });
            jobs.push(Job { hero: None, mon: Some(i), a: Brain::Scripted, b: brain_m(i), seed: next_seed() });
        }
    }
    for _ in 0..hof {
        for i in 0..n {
            let m = hof_m.pick(rng).clone();
            jobs.push(Job { hero: Some(i), mon: None, a: brain_h(i), b: Brain::Net(m), seed: next_seed() });
            let h = hof_h.pick(rng).clone();
            jobs.push(Job { hero: None, mon: Some(i), a: Brain::Net(h), b: brain_m(i), seed: next_seed() });
        }
    }
    jobs
}

fn breed(pop: &mut Vec<Genome>, fit: &[f32], cfg: &TrainConfig, gen: u32, rng: &mut Rng) {
    let n = pop.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| fit[b].partial_cmp(&fit[a]).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(&b)));
    let frac = if cfg.gens > 1 { gen as f32 / (cfg.gens - 1) as f32 } else { 1.0 };
    let sigma = cfg.sigma_start + (cfg.sigma_end - cfg.sigma_start) * frac;
    let mut next: Vec<Genome> = Vec::with_capacity(n);
    for &i in idx.iter().take(cfg.elites) {
        let mut g = pop[i].clone();
        g.fitness = fit[i];
        next.push(g);
    }
    let pick = |rng: &mut Rng| -> usize {
        let mut best = (rng.next_u64() % n as u64) as usize;
        for _ in 1..cfg.tournament {
            let c = (rng.next_u64() % n as u64) as usize;
            if fit[c] > fit[best] {
                best = c;
            }
        }
        best
    };
    while next.len() < n {
        let a = pick(rng);
        let mut child = if rng.f32() < 0.5 {
            let b = pick(rng);
            Genome::crossover(&pop[a], &pop[b], rng)
        } else {
            pop[a].clone()
        };
        child.mutate(rng, cfg.mutation_rate, sigma);
        child.generation = gen + 1;
        child.fitness = 0.0;
        next.push(child);
    }
    *pop = next;
}

fn civil_date() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let z = secs / 86400 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn io(e: std::io::Error) -> String {
    e.to_string()
}

/// Train both populations; writes training.csv, champions, fight logs and run.json into `cfg.out`.
pub fn train(cfg: &TrainConfig) -> Result<(), String> {
    std::fs::create_dir_all(&cfg.out).map_err(io)?;
    let mut rng = Rng::new(cfg.seed);
    let mut heroes: Vec<Genome> = (0..cfg.pop).map(|_| Genome::random(Role::Hero, &mut rng)).collect();
    let mut mons: Vec<Genome> = (0..cfg.pop).map(|_| Genome::random(Role::Monster, &mut rng)).collect();
    let mut hof_h = HallOfFame::new(8);
    let mut hof_m = HallOfFame::new(8);
    let mut csv = csv_header(&["gen", "stage", "hero_best", "hero_mean", "monster_best", "monster_mean", "defence_rate", "idle_guard", "draw_share", "mean_ticks"]);
    let mut fight_gens: Vec<u32> = Vec::new();
    let mut evals: Vec<String> = Vec::new();
    for gen in 0..cfg.gens {
        let stage = stage_for_gen(gen);
        let jobs = make_jobs(gen, stage, &heroes, &mons, &hof_h, &hof_m, cfg.seed, &mut rng);
        let results = run_jobs(&jobs, cfg.threads);
        let mut sum = [vec![0.0f32; cfg.pop], vec![0.0f32; cfg.pop]];
        let mut cnt = [vec![0u32; cfg.pop], vec![0u32; cfg.pop]];
        let (mut def, mut threats, mut idle, mut draws, mut ticks) = (0u32, 0u32, 0.0f32, 0u32, 0.0f32);
        for (j, r) in jobs.iter().zip(results.iter()) {
            if let Some(i) = j.hero {
                sum[0][i] += score_match(r, 0);
                cnt[0][i] += 1;
            }
            if let Some(i) = j.mon {
                sum[1][i] += score_match(r, 1);
                cnt[1][i] += 1;
            }
            for d in &r.defence {
                def += d.parry + d.block + d.dodge;
                threats += d.parry + d.block + d.dodge + d.hit + d.miss;
            }
            idle += (r.idle_guard_share[0] + r.idle_guard_share[1]) * 0.5;
            if r.outcome == Outcome::Draw {
                draws += 1;
            }
            ticks += r.ticks as f32;
        }
        let nj = jobs.len().max(1) as f32;
        let fit: Vec<Vec<f32>> = (0..2).map(|s| (0..cfg.pop).map(|i| sum[s][i] / cnt[s][i].max(1) as f32).collect()).collect();
        let best = |v: &Vec<f32>| v.iter().cloned().fold(f32::MIN, f32::max);
        let mean = |v: &Vec<f32>| v.iter().sum::<f32>() / v.len() as f32;
        let drate = if threats > 0 { def as f32 / threats as f32 } else { 0.0 };
        let stage_name = match stage {
            Stage::A => "A",
            Stage::B => "B",
            Stage::C => "C",
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            gen, stage_name, best(&fit[0]), mean(&fit[0]), best(&fit[1]), mean(&fit[1]), drate, idle / nj, draws as f32 / nj, ticks / nj
        ));
        let champ = |pop: &Vec<Genome>, f: &Vec<f32>| -> Genome {
            let mut bi = 0;
            for i in 1..pop.len() {
                if f[i] > f[bi] {
                    bi = i;
                }
            }
            let mut g = pop[bi].clone();
            g.generation = gen;
            g.fitness = f[bi];
            g
        };
        let last = gen + 1 == cfg.gens;
        if hof_h.should_save(gen) || last || gen == 0 {
            let (ch, cm) = (champ(&heroes, &fit[0]), champ(&mons, &fit[1]));
            if hof_h.should_save(gen) {
                hof_h.push(ch.clone());
                hof_m.push(cm.clone());
            }
            let w = |name: String, body: String| std::fs::write(cfg.out.join(name), body).map_err(io);
            w(format!("champion_hero_gen_{}.json", gen), ch.to_json())?;
            w(format!("champion_monster_gen_{}.json", gen), cm.to_json())?;
            let mut m = Match::new(cfg.seed.wrapping_add(gen as u64), Brain::Net(ch), Brain::Net(cm));
            m.enable_log();
            m.run_to_end();
            w(format!("fight_gen_{}.json", gen), crate::log::fight_json(&m))?;
            fight_gens.push(gen);
