//! Genome (WS2 minimal; WS4 owns JSON parsing and any richer encoding).
use crate::robot::Role;
use crate::net::N_PARAMS;
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Genome {
    pub role: Role,
    pub weights: Vec<f32>,
    pub generation: u32,
    pub fitness: f32,
}
