// Style teachers, imitation and style training (plan 2026-10-25_v3-masters-overnight.md, first stage).
use mech_sim::arena::Arena;
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::genome::Genome;
use mech_sim::imitate::{distil, DistilOpts};
use mech_sim::robot::Role;
use mech_sim::rng::Rng;
use mech_sim::stylebot::StyleId;
use mech_sim::stylematrix::run_pair;

fn flat_win_rate(a: Brain, b: Brain, seeds: u64) -> f64 {
    let (x, _) = run_pair(&a, &b, seeds, true);
    x.win_rate()
}
