// r7: a sword that is not striking cannot rest inside the opponent's body (same rule both sides).
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::stylebot::StyleId;

fn net(path: &str) -> Option<Genome> {
    let p = format!("{}/../runs/{}", env!("CARGO_MANIFEST_DIR"), path);
    std::fs::read_to_string(p).ok().and_then(|s| Genome::from_json(&s).ok())
}
