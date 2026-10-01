// W3 duel loop: outcomes, scripted opponents, anti-stalemate bleed.
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::robot::{Part, GUARD_TARGETS};

fn total_hp(m: &Match, who: usize) -> f32 {
    Part::ALL.iter().map(|p| m.robots[who].hp(*p).max(0.0)).sum()
}

#[test]
fn start_distance_is_5_m_in_14_m_arena() {
    let m = Match::new(1, Brain::Dummy, Brain::Dummy);
    assert_eq!(m.arena.width, 14.0);
    assert!(((m.robots[1].x - m.robots[0].x).abs() - 5.0).abs() < 0.2);
    assert_eq!(m.robots[0].facing, 1.0);
    assert_eq!(m.robots[1].facing, -1.0);
}
