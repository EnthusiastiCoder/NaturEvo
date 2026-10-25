//! Mutual stagger must not be a stable state: clashes cannot keep both fighters down for a large share of a fight.
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::robot::State;

/// (ticks both staggered, total ticks, clash-free-hit count proxy: ticks) over `seeds` fights.
fn both_staggered(a: Brain, b: Brain, seeds: u64) -> (u32, u32) {
    let (mut both, mut n) = (0u32, 0u32);
    for seed in 0..seeds {
        let mut m = Match::new(seed, a.clone(), b.clone());
        while !m.is_over() {
            m.step();
            n += 1;
            both += m.robots.iter().all(|r| r.state() == State::Staggered) as u32;
        }
    }
    (both, n)
}

fn load(path: &str) -> Option<Genome> {
    let p = format!("{}/../runs/{}", env!("CARGO_MANIFEST_DIR"), path);
    std::fs::read_to_string(p).ok().and_then(|s| Genome::from_json(&s).ok())
}
