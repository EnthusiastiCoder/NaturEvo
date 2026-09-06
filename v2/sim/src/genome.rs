//! Genome (WS2 minimal; WS4 owns JSON parsing and any richer encoding).
use crate::fighter::Role;
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
        Ok(Genome { role, weights, generation, fitness })
    }
}

/// Raw value text of a top-level key (flat object, array values contain no nesting).
fn field<'a>(s: &'a str, key: &str) -> Result<&'a str, String> {
    let pat = format!("\"{key}\":");
    let start = s.find(&pat).ok_or_else(|| format!("missing {key}"))? + pat.len();
    let rest = s[start..].trim_start();
    let end = if rest.starts_with('[') {
        rest.find(']').ok_or("unterminated array")? + 1
    } else {
        rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len())
    };
    Ok(rest[..end].trim())
}
