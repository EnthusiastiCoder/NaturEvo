// W3 fuzz: 200 random-genome death-battle matches stay finite, in range and never panic.
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::robot::{Part, Role};
use mech_sim::rng::Rng;

#[test]
fn two_hundred_random_matches_stay_sane() {
    for seed in 0..200u64 {
        let mut rng = Rng::new(seed * 7 + 1);
        let a = Genome::random(Role::Hero, &mut rng);
        let b = Genome::random(Role::Monster, &mut rng);
        let mut m = Match::new(seed, Brain::Net(a), Brain::Net(b));
        let r = m.run_to_end();
        assert!(!r.capped && r.ticks < mech_sim::config::HARD_CAP_TICKS, "seed {}", seed);
        for who in 0..2 {
            let rb = &m.robots[who];
            assert!(rb.x.is_finite() && rb.com()[0].is_finite() && rb.com()[1].is_finite(), "seed {} pos", seed);
            assert!(rb.x >= 0.0 && rb.x <= m.arena.width, "seed {} left the arena: {}", seed, rb.x);
            assert!((0.0..=100.0).contains(&rb.energy), "seed {} energy {}", seed, rb.energy);
            assert!((0.0..=100.0).contains(&rb.posture), "seed {} posture {}", seed, rb.posture);
            for p in Part::ALL {
                let hp = rb.hp(p);
                assert!(hp.is_finite(), "seed {}", seed);
                assert!(hp <= rb.max_hp(p) + 1e-3 && hp >= -rb.max_hp(p), "seed {} {:?} hp {}", seed, p, hp);
                let c = rb.capsule(p);
                assert!(c.a.iter().chain(c.b.iter()).all(|v| v.is_finite()), "seed {} capsule", seed);
            }
        }
    }
}
