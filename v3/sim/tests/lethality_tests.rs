//! Lethality model: no single hit ends a duel, a destroyed head blinds instead of killing,
//! death is torso destroyed or the part-health pool below KO_POOL_FRAC, duels last.
mod common;
use common::*;
use mech_sim::config::IMPACT_CAP;
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::robot::{Part, Role, State, KO_POOL_FRAC};

#[test]
fn no_single_hit_from_full_health_is_ko() {
    for p in Part::ALL {
        for exposed in [false, true] {
            let mut r = standing(Role::Hero, 3.0, 1.0);
            if exposed {
                r.add_posture(100.0);
            }
            r.damage_part(p, IMPACT_CAP);
            assert!(!r.is_ko(), "{:?} exposed={} died from one hit", p, exposed);
        }
    }
}

#[test]
fn a_destroyed_head_is_not_death() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.set_hp(Part::Head, -0.5 * r.max_hp(Part::Head));
    assert!(r.is_destroyed(Part::Head) && r.is_disabled(Part::Head));
    assert!(!r.is_ko());
    r.apply_targets(&mech_sim::robot::STAND_TARGETS, mech_sim::robot::DT);
    assert_ne!(r.state(), State::Ko);
    assert!(r.can_act() || r.state() != State::Ko);
}

#[test]
fn torso_destroyed_is_death() {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    r.set_hp(Part::Torso, -0.5 * r.max_hp(Part::Torso));
    assert!(r.is_ko());
}

#[test]
fn pool_below_fraction_is_death_and_above_is_not() {
    let total: f32 = Part::ALL.iter().map(|p| standing(Role::Hero, 3.0, 1.0).max_hp(*p)).sum();
    let mut r = standing(Role::Hero, 3.0, 1.0);
    // Strip every non-torso part to zero hp, then leave the torso at a given pool fraction.
    for p in Part::ALL {
        if p != Part::Torso {
            r.set_hp(p, 0.0);
        }
    }
    let torso = r.max_hp(Part::Torso);
    r.set_hp(Part::Torso, (KO_POOL_FRAC * total + 3.0).min(torso));
    if KO_POOL_FRAC * total + 3.0 < torso {
        assert!(!r.is_ko(), "just above the pool fraction must live");
    }
    r.set_hp(Part::Torso, KO_POOL_FRAC * total - 3.0);
    assert!(r.is_ko(), "just below the pool fraction must die");
}

#[test]
fn many_scripted_duels_are_long_and_decisive() {
    let mut ticks = Vec::new();
    let mut draws = 0;
    for seed in 1..=8u64 {
        let r = Match::new(seed, Brain::Scripted, Brain::Scripted).run_to_end();
        assert!(r.ticks >= 150, "seed {} ended after {} ticks", seed, r.ticks);
        draws += (r.outcome == Outcome::Draw) as u32;
        assert!(r.hits[0] + r.hits[1] >= 3, "seed {} only {} hits", seed, r.hits[0] + r.hits[1]);
        ticks.push(r.ticks);
    }
    assert!(draws <= 2, "{} of 8 scripted duels undecided", draws);
    ticks.sort();
    assert!(ticks[ticks.len() / 2] >= 300, "median {:?}", ticks);
}

#[test]
fn trained_champions_average_at_least_five_hits_per_fight() {
    let runs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../runs");
    let mut tot = 0u32;
    let mut n = 0u32;
    for s in 1..=3 {
        let dir = runs.join(format!("slice_s{}", s));
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        let last = |role: &str| {
            rd_last(&dir, role)
        };
        let _ = rd;
        let (Some(h), Some(m)) = (last("hero"), last("monster")) else { continue };
        for seed in 0..4u64 {
            let r = Match::new(1000 + seed, Brain::Net(h.clone()), Brain::Net(m.clone())).run_to_end();
            tot += r.hits[0] + r.hits[1];
            n += 1;
        }
    }
    if n == 0 {
        eprintln!("no trained runs present; skipped");
        return;
    }
    assert!(tot as f32 / n as f32 >= 5.0, "{} hits over {} fights", tot, n);
}
