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
