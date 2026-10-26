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

#[test]
fn teachers_beat_dummy_on_flat_arena() {
    for st in StyleId::ALL {
        let w = flat_win_rate(Brain::Style(st), Brain::Dummy, 20);
        assert!(w >= 0.9, "{} beat the dummy in {:.2}", st.name(), w);
    }
}

#[test]
fn teachers_are_decisive_against_skill2() {
    for st in StyleId::ALL {
        let (x, _) = run_pair(&Brain::Style(st), &Brain::Skill(2), 20, true);
        assert!(x.win_rate() >= 0.6, "{} vs skill2 {:.2}", st.name(), x.win_rate());
        assert!(x.capped_share() <= 0.2, "{} capped {:.2}", st.name(), x.capped_share());
    }
}
