// W3 duel loop: outcomes, scripted opponents, anti-stalemate bleed.
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::robot::{Part, GUARD_TARGETS};

fn total_hp(m: &Match, who: usize) -> f32 {
    Part::ALL.iter().map(|p| m.robots[who].hp(*p).max(0.0)).sum()
}
