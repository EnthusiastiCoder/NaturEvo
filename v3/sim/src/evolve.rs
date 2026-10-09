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
