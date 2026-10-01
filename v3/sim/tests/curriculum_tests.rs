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
