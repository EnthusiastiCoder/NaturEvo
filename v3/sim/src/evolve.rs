//! Co-evolution of hero and monster populations with curriculum, hall of fame and threaded evaluation.
use crate::curriculum::{hof_enabled, opponent_mix, pick_kind, stage_for_gen, OppKind, Stage};
use crate::duel::{Brain, Match, MatchResult, Outcome};
use crate::fitness::score_match;
use crate::genome::Genome;
use crate::metrics::csv_header;
use crate::rng::Rng;
use crate::robot::Role;
use crate::stylebot::StyleId;
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
    /// Style training: teacher style, ego-frame seed genome (distilled) and frozen ego-frame foes from other styles.
    pub style: Option<StyleId>,
    pub seed_genome: Option<Genome>,
    pub foes: Vec<Genome>,
}

impl TrainConfig {
    pub fn new(seed: u64, pop: usize, gens: u32, out: PathBuf) -> TrainConfig {
        let name = out.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "run".into());
        TrainConfig { seed, pop, gens, out, threads: 1, elites: (pop / 16).max(1), tournament: 3, mutation_rate: 0.08, sigma_start: 0.25, sigma_end: 0.05, name, style: None, seed_genome: None, foes: Vec::new() }
    }
}

const OPPONENTS_PER_INDIVIDUAL: usize = 2;

struct Job {
    hero: Option<usize>,
    mon: Option<usize>,
    a: Brain,
    b: Brain,
    seed: u64,
    flat: bool,
}

/// Run all jobs; threads pull the next job from a shared counter (fights vary a lot in length), results are
/// stored by job index so the outcome is identical for any thread count.
fn run_jobs(jobs: &[Job], threads: usize) -> Vec<MatchResult> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    let n = jobs.len();
    let t = threads.max(1).min(n.max(1));
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<MatchResult>>> = Mutex::new((0..n).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..t {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                let j = &jobs[i];
                let mut m = Match::new(j.seed, j.a.clone(), j.b.clone());
                m.disable_grace_trace();
                let r = m.run_to_end();
                out.lock().unwrap()[i] = Some(r);
            });
        }
    });
    out.into_inner().unwrap().into_iter().map(|o| o.expect("job result")).collect()
}

#[allow(dead_code)]
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
    // Two sampled opponents per individual per side keeps 1000 gens x pop 64 under 10 minutes; the mix supplies variety.
    let k = OPPONENTS_PER_INDIVIDUAL;
    let mix = opponent_mix(gen);
    let use_hof = hof_enabled(stage) && !hof_h.is_empty() && !hof_m.is_empty();
    for hero_side in [true, false] {
        for i in 0..n {
            for _ in 0..k {
                let mut kind = pick_kind(&mix, rng.f32());
                if kind == OppKind::Hof && !use_hof {
                    kind = OppKind::SelfPlay;
                }
                let opp = match kind {
                    OppKind::Dummy => Brain::Dummy,
                    OppKind::Skill(t) => Brain::Skill(t),
                    OppKind::SelfPlay => {
                        let j = (rng.next_u64() % n as u64) as usize;
                        if hero_side { Brain::Net(mons[j].clone()) } else { Brain::Net(heroes[j].clone()) }
                    }
                    OppKind::Hof => {
                        if hero_side { Brain::Net(hof_m.pick(rng).clone()) } else { Brain::Net(hof_h.pick(rng).clone()) }
                    }
                };
                let sd = next_seed();
                jobs.push(if hero_side {
                    Job { hero: Some(i), mon: None, a: brain_h(i), b: opp, seed: sd }
                } else {
                    Job { hero: None, mon: Some(i), a: opp, b: brain_m(i), seed: sd }
                });
            }
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
    let mut csv = csv_header(&["gen", "stage", "hero_best", "hero_mean", "monster_best", "monster_mean", "defence_rate", "idle_guard", "draw_share", "mean_ticks", "capped_share", "jerk"]);
    let mut fight_gens: Vec<u32> = Vec::new();
    let mut evals: Vec<String> = Vec::new();
    for gen in 0..cfg.gens {
        let stage = stage_for_gen(gen);
        let jobs = make_jobs(gen, stage, &heroes, &mons, &hof_h, &hof_m, cfg.seed, &mut rng);
        let results = run_jobs(&jobs, cfg.threads);
        let mut sum = [vec![0.0f32; cfg.pop], vec![0.0f32; cfg.pop]];
        let mut cnt = [vec![0u32; cfg.pop], vec![0u32; cfg.pop]];
        let (mut def, mut threats, mut idle, mut draws, mut ticks) = (0u32, 0u32, 0.0f32, 0u32, 0.0f32);
        let (mut capped, mut jerk) = (0u32, 0.0f32);
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
            capped += r.capped as u32;
            jerk += (r.jerk[0] + r.jerk[1]) * 0.5;
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
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            gen, stage_name, best(&fit[0]), mean(&fit[0]), best(&fit[1]), mean(&fit[1]), drate, idle / nj, draws as f32 / nj, ticks / nj, capped as f32 / nj, jerk / nj
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
            evals.push(format!("{{\"gen\":{},\"defence_rate\":{},\"idle_guard\":{},\"draw_share\":{}}}", gen, drate, idle / nj, draws as f32 / nj));
        }
        eprintln!(
            "gen {} stage {} hero {:.2}/{:.2} mon {:.2}/{:.2} def {:.3} idle {:.3} draw {:.2}",
            gen, stage_name, best(&fit[0]), mean(&fit[0]), best(&fit[1]), mean(&fit[1]), drate, idle / nj, draws as f32 / nj
        );
        let mut r1 = Rng::new(cfg.seed ^ 0xA5A5 ^ ((gen as u64) << 8));
        breed(&mut heroes, &fit[0], cfg, gen, &mut r1);
        breed(&mut mons, &fit[1], cfg, gen, &mut r1);
    }
    std::fs::write(cfg.out.join("training.csv"), csv).map_err(io)?;
    let fg: Vec<String> = fight_gens.iter().map(|g| g.to_string()).collect();
    let run = format!(
        "{{\"name\":\"{}\",\"seed\":{},\"date\":\"{}\",\"population\":{},\"generations\":{},\"fight_gens\":[{}],\"config\":{{\"elites\":{},\"tournament\":{},\"mutation_rate\":{},\"sigma_start\":{},\"sigma_end\":{}}},\"evals\":[{}]}}",
        cfg.name, cfg.seed, civil_date(), cfg.pop, cfg.gens, fg.join(","), cfg.elites, cfg.tournament, cfg.mutation_rate, cfg.sigma_start, cfg.sigma_end, evals.join(",")
    );
    std::fs::write(cfg.out.join("run.json"), run).map_err(io)?;
    Ok(())
}
