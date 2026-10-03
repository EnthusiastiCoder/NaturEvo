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

    /// Uniform crossover; child takes a's role and generation = max + 1.
    pub fn crossover(a: &Genome, b: &Genome, rng: &mut Rng) -> Genome {
        let n = a.weights.len().min(b.weights.len());
        let weights = (0..n)
            .map(|i| if rng.f32() < 0.5 { a.weights[i] } else { b.weights[i] })
            .collect();
        Genome {
            role: a.role,
            weights,
            generation: a.generation.max(b.generation) + 1,
            fitness: 0.0,
        }
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

    pub fn from_json(s: &str) -> Result<Genome, String> {
        let s = s.trim();
        if !s.starts_with('{') || !s.ends_with('}') {
            return Err("not a JSON object".into());
        }
        let role = match field(s, "role")?.trim_matches('"') {
            "hero" => Role::Hero,
            "monster" => Role::Monster,
            r => return Err(format!("bad role {r}")),
        };
        let generation = field(s, "generation")?.parse::<u32>().map_err(|e| e.to_string())?;
        let fitness = field(s, "fitness")?.parse::<f32>().map_err(|e| e.to_string())?;
        let w = field(s, "weights")?;
        let inner = w.strip_prefix('[').and_then(|x| x.strip_suffix(']')).ok_or("bad weights")?;
        let weights = if inner.trim().is_empty() {
            Vec::new()
        } else {
            inner
                .split(',')
                .map(|v| v.trim().parse::<f32>().map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?
        };
        if weights.iter().any(|w| !w.is_finite()) {
            return Err("non-finite weight".into());
        }
        if weights.len() != N_WEIGHTS {
            return Err(format!("weights length {} does not match expected {}", weights.len(), N_WEIGHTS));
        }
        Ok(Genome { role, weights, generation, fitness })
    }
}
