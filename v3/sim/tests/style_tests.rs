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

#[test]
fn style_metrics_differ() {
    let m = |st: StyleId| {
        let (x, _) = run_pair(&Brain::Style(st), &Brain::Skill(2), 20, true);
        (x.hits_per_swing(), x.combo(), x.mean_ticks())
    };
    let (p, c, d) = (m(StyleId::Press), m(StyleId::Counter), m(StyleId::Duelist));
    let spread = |f: &dyn Fn(&(f64, f64, f64)) -> f64| {
        let v = [f(&p), f(&c), f(&d)];
        v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
    };
    assert!(spread(&|t| t.0) > 0.2 || spread(&|t| t.1) > 0.3 || spread(&|t| t.2) > 100.0, "styles are not distinguishable: {:?} {:?} {:?}", p, c, d);
}

#[test]
fn teacher_fights_are_deterministic() {
    let run = || {
        let mut m = Match::new(7, Brain::Style(StyleId::Press), Brain::Style(StyleId::Counter));
        m.arena = Arena::flat(m.arena.width);
        let r = m.run_to_end();
        (r.ticks, r.outcome, r.hits, r.swings)
    };
    assert_eq!(run(), run());
}

#[test]
fn mirrored_drive_roundtrips_and_leaves_hero_alone() {
    let g = Genome::random(Role::Hero, &mut Rng::new(3));
    assert_eq!(g.mirrored_drive(Role::Hero).weights, g.weights);
    let m = g.mirrored_drive(Role::Monster);
    assert_ne!(m.weights, g.weights);
    assert_eq!(m.mirrored_drive(Role::Monster).weights, g.weights);
}

#[test]
fn distillation_is_accurate_and_deterministic() {
    let o = DistilOpts { per_opp: 4, rounds: 1, epochs: 12, lr: 0.004, seed: 5 };
    let (g1, s1) = distil(StyleId::Press, &o);
    let (g2, _) = distil(StyleId::Press, &o);
    assert_eq!(g1.weights, g2.weights, "distillation must be deterministic");
    assert!(s1.active_acc > 0.85, "active_acc {}", s1.active_acc);
    assert!(s1.move_acc > 0.85, "move_acc {}", s1.move_acc);
    // The student must at least walk to the dummy and not stall.
    let mut m = Match::new(1, Brain::Net(g1.clone()), Brain::Dummy);
    m.arena = Arena::flat(m.arena.width);
    let r = m.run_to_end();
    assert!(r.swings[0] > 0 || r.outcome != Outcome::Draw, "student never engaged");
}

#[test]
fn style_training_runs_and_is_thread_independent() {
    use mech_sim::evolve::{train, TrainConfig};
    let (seed, _) = distil(StyleId::Counter, &DistilOpts { per_opp: 2, rounds: 0, epochs: 4, lr: 0.004, seed: 2 });
    let run = |threads: usize, name: &str| {
        let out = std::env::temp_dir().join(format!("style_train_{}_{}", std::process::id(), name));
        let mut cfg = TrainConfig::new(3, 8, 3, out.clone());
        cfg.threads = threads;
        cfg.style = Some(StyleId::Counter);
        cfg.seed_genome = Some(seed.clone());
        cfg.foes = vec![seed.clone()];
        train(&cfg).unwrap();
        let t = std::fs::read_to_string(out.join("champion_hero_gen_2.json")).unwrap();
        let _ = std::fs::remove_dir_all(&out);
        t
    };
    assert_eq!(run(1, "a"), run(3, "b"));
}
