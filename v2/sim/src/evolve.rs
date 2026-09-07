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
