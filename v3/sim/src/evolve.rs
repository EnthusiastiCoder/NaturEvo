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
