//! Held-out evaluation of a run's final champions, and the aggregate over several training seeds.
use crate::duel::{Brain, Match, Outcome};
use crate::genome::Genome;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct EvalMetrics {
    pub matches: u32,
    pub win_rate_hof: f32,
    /// (parried + blocked + dodged) / live threats, by the final champions against earlier champions.
    pub defence_rate: f32,
    pub defence_rate_scripted: f32,
    pub hit_rate: f32,
    pub idle_guard: f32,
    pub draw_share: f32,
    pub mean_ticks: f32,
    pub parry_share: f32,
    pub block_share: f32,
    pub dodge_share: f32,
}
