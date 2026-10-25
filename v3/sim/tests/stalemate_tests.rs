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

#[test]
fn skillbots_are_rarely_both_staggered() {
    for tier in [1u8, 2, 3] {
        let (both, n) = both_staggered(Brain::Skill(tier), Brain::Skill(tier), 6);
        println!("tier {tier}: both staggered {both}/{n}");
        assert!((both as f32) < 0.30 * n as f32, "tier {tier}: {both}/{n} ticks both staggered");
    }
}

#[test]
fn trained_nets_are_not_stuck_in_mutual_stagger() {
    for run in ["slice_s1", "slice_s2", "slice_s3"] {
        let (Some(h), Some(mo)) = (load(&format!("{run}/champion_hero_gen_999.json")), load(&format!("{run}/champion_monster_gen_999.json"))) else {
            continue;
        };
        let (both, n) = both_staggered(Brain::Net(h), Brain::Net(mo), 4);
        println!("{run}: both staggered {both}/{n}");
        assert!((both as f32) < 0.10 * n as f32, "{run}: {both}/{n} ticks both staggered");
    }
}
