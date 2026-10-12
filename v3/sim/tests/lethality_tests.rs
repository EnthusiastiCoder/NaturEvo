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
