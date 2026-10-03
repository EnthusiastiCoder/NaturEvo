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

    /// Gaussian-ish (sum of uniforms) perturbation of each weight with probability `rate`.
    pub fn mutate(&mut self, rng: &mut Rng, rate: f32, sigma: f32) {
        for w in self.weights.iter_mut() {
            if rng.f32() < rate {
                let n = rng.f32() + rng.f32() + rng.f32() + rng.f32() - 2.0;
                *w = (*w + n * 1.7320508 * sigma).clamp(-W_CLAMP, W_CLAMP);
            }
        }
    }
