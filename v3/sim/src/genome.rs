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

pub const N_WEIGHTS: usize = N_PARAMS;
pub const W_CLAMP: f32 = 8.0;

impl Genome {
    pub fn random(role: Role, rng: &mut Rng) -> Genome {
        let weights = (0..N_WEIGHTS).map(|_| rng.range(-0.5, 0.5)).collect();
        Genome { role, weights, generation: 0, fitness: 0.0 }
    }
