// W4 curriculum stages and hall of fame.
use mech_sim::curriculum::{hof_enabled, opponents_per_individual, stage_for_gen, Stage};
use mech_sim::evolve::HallOfFame;
use mech_sim::genome::Genome;
use mech_sim::rng::Rng;
use mech_sim::robot::Role;

#[test]
fn stage_by_generation() {
    assert_eq!(stage_for_gen(0), Stage::A);
    assert_eq!(stage_for_gen(29), Stage::A);
    assert_eq!(stage_for_gen(30), Stage::B);
    assert_eq!(stage_for_gen(79), Stage::B);
    assert_eq!(stage_for_gen(80), Stage::C);
    assert_eq!(stage_for_gen(200), Stage::C);
}

#[test]
fn hof_only_in_stage_c() {
    assert!(!hof_enabled(Stage::A));
    assert!(!hof_enabled(Stage::B));
    assert!(hof_enabled(Stage::C));
}

#[test]
fn opponents_per_individual_is_2_in_a_then_3() {
    assert_eq!(opponents_per_individual(Stage::A), 2);
    assert_eq!(opponents_per_individual(Stage::B), 3);
    assert_eq!(opponents_per_individual(Stage::C), 3);
}

#[test]
fn hof_saves_every_10_gens_and_keeps_last_8() {
    let mut h = HallOfFame::new(8);
    assert!(h.should_save(10) && h.should_save(120));
    assert!(!h.should_save(9) && !h.should_save(15));
    for gen in (10..=120).step_by(10) {
        let mut g = Genome::random(Role::Hero, &mut Rng::new(gen as u64));
        g.generation = gen;
        h.push(g);
    }
    assert_eq!(h.len(), 8);
    assert_eq!(h.generations(), vec![50, 60, 70, 80, 90, 100, 110, 120]);
}

#[test]
fn hof_pick_is_uniform_over_members_and_deterministic() {
    let mut h = HallOfFame::new(8);
    for gen in [10u32, 20, 30] {
        let mut g = Genome::random(Role::Hero, &mut Rng::new(gen as u64));
        g.generation = gen;
        h.push(g);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut r = Rng::new(1);
    for _ in 0..200 {
        seen.insert(h.pick(&mut r).generation);
    }
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![10, 20, 30]);
    let (mut r1, mut r2) = (Rng::new(5), Rng::new(5));
    assert_eq!(h.pick(&mut r1).generation, h.pick(&mut r2).generation);
}
