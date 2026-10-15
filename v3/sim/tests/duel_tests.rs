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
fn dummy_vs_dummy_ends_in_a_death_after_the_grace_period_not_a_timeout() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let r = m.run_to_end();
    assert!(r.ticks > mech_sim::config::SUDDEN_DEATH_START && !r.capped);
    assert!(m.robots.iter().any(|x| x.is_ko()));
}

#[test]
fn scripted_attacker_damages_a_standing_fighter_who_does_nothing() {
    let mut m = Match::new(3, Brain::Scripted, Brain::Dummy);
    let before = total_hp(&m, 1);
    let r = m.run_to_end();
    assert!(total_hp(&m, 1) < before - 1.0, "dummy lost hp: {} -> {}", before, total_hp(&m, 1));
    assert!(!r.capped);
}

#[test]
fn match_ends_only_with_a_death() {
    let mut m = Match::new(5, Brain::Scripted, Brain::Dummy);
    let r = m.run_to_end();
    let ko = m.robots.iter().any(|x| x.is_ko());
    assert!(ko && !r.capped, "ended at {} without KO", r.ticks);
    if ko {
        assert_ne!(r.outcome, Outcome::Draw);
    }
}

#[test]
fn sudden_death_bleed_starts_only_at_the_grace_period_end() {
    let mut m = Match::new(1, Brain::Dummy, Brain::Dummy);
    let max = m.robots[0].max_hp(Part::Torso);
    while m.tick < mech_sim::config::SUDDEN_DEATH_START - 5 {
        m.step();
    }
    assert_eq!(m.robots[0].hp(Part::Torso), max);
    assert_eq!(m.robots[1].hp(Part::Torso), max);
    while m.tick < mech_sim::config::SUDDEN_DEATH_START + 60 {
        m.step();
    }
    for i in 0..2 {
        let lost = max - m.robots[i].hp(Part::Torso);
        assert!(lost > 2.0 && lost < 6.0, "fighter {} lost {} torso hp in 1 s of sudden death", i, lost);
    }
}

#[test]
fn guard_pose_held_without_threat_is_idle_guard_and_drains_energy() {
    let mut m = Match::new(1, Brain::Fixed(GUARD_TARGETS), Brain::Dummy);
    for _ in 0..300 {
        m.step();
    }
    assert!(m.robots[0].energy < 80.0, "energy {}", m.robots[0].energy);
    assert!(m.idle_guard_ticks[0] > 270, "idle guard ticks {}", m.idle_guard_ticks[0]);
    assert_eq!(m.idle_guard_ticks[1], 0);
}
