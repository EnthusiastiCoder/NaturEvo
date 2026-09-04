//! Genome (WS2 minimal; WS4 owns JSON parsing and any richer encoding).
use crate::fighter::Role;
use crate::net::N_OUT;
use crate::rng::Rng;
use crate::sense::N_IN;

#[derive(Clone, Debug)]
pub struct Genome {
    pub role: Role,
    pub weights: Vec<f32>,
    pub generation: u32,
    pub fitness: f32,
}

pub const N_WEIGHTS: usize = N_IN * N_OUT + N_OUT;

impl Genome {
    pub fn random(role: Role, rng: &mut Rng) -> Genome {
        let weights = (0..N_WEIGHTS).map(|_| rng.range(-1.0, 1.0)).collect();
        Genome { role, weights, generation: 0, fitness: 0.0 }
    }

    pub fn to_json(&self) -> String {
        let w: Vec<String> = self.weights.iter().map(|v| format!("{}", v)).collect();
        format!(
            "{{\"role\":\"{}\",\"generation\":{},\"fitness\":{},\"weights\":[{}]}}",
            self.role.as_str(),
            self.generation,
            self.fitness,
            w.join(",")
        )
    }
