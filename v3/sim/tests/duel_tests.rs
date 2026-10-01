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

#[test]
fn dummy_vs_dummy_times_out_as_draw_at_900_ticks() {
    let r = Match::new(1, Brain::Dummy, Brain::Dummy).run_to_end();
    assert_eq!(r.ticks, 900);
    assert_eq!(r.outcome, Outcome::Draw);
}

#[test]
fn scripted_attacker_damages_a_standing_fighter_who_does_nothing() {
    let mut m = Match::new(3, Brain::Scripted, Brain::Dummy);
    let before = total_hp(&m, 1);
    let r = m.run_to_end();
    assert!(total_hp(&m, 1) < before - 1.0, "dummy lost hp: {} -> {}", before, total_hp(&m, 1));
    assert!(r.ticks <= 900);
}

#[test]
fn match_ends_with_ko_or_timeout() {
    let mut m = Match::new(5, Brain::Scripted, Brain::Dummy);
    let r = m.run_to_end();
    let ko = m.robots.iter().any(|x| x.is_ko());
    assert!(ko || r.ticks == 900, "ended at {} without KO", r.ticks);
    if ko {
        assert_ne!(r.outcome, Outcome::Draw);
    }
}

#[test]
fn anti_stalemate_bleed_only_after_tick_450_at_half_hp_per_second() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let max = m.robots[0].max_hp(Part::Torso);
    for _ in 0..440 {
        m.step();
    }
    assert_eq!(m.robots[0].hp(Part::Torso), max);
    assert_eq!(m.robots[1].hp(Part::Torso), max);
    while m.tick < 600 {
        m.step();
    }
    for i in 0..2 {
        let lost = max - m.robots[i].hp(Part::Torso);
        assert!((1.0..=1.5).contains(&lost), "fighter {} lost {} torso hp in 150 ticks (expect ~1.25)", i, lost);
    }
}
