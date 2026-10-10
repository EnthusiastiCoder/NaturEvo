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

fn champions(dir: &Path, role: &str) -> Result<Vec<Genome>, String> {
    let prefix = format!("champion_{}_gen_", role);
    let mut found: Vec<(u32, Genome)> = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {}", dir.display(), e))? {
        let name = e.map_err(|e| e.to_string())?.file_name().to_string_lossy().to_string();
        if let Some(rest) = name.strip_prefix(&prefix).and_then(|r| r.strip_suffix(".json")) {
            if let Ok(g) = rest.parse::<u32>() {
                let text = std::fs::read_to_string(dir.join(&name)).map_err(|e| e.to_string())?;
                found.push((g, Genome::from_json(&text)?));
            }
        }
    }
    found.sort_by_key(|f| f.0);
    Ok(found.into_iter().map(|f| f.1).collect())
}
