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
