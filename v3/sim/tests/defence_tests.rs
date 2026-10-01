// W3 defence: live-threat detection and exactly-once resolution.
mod common;
use common::{standing, torso_center};
use mech_sim::duel::{is_live_threat, Brain, Match, Resolution};
use mech_sim::robot::{Role, GUARD_TARGETS};

#[test]
fn fast_blade_heading_to_body_is_a_live_threat() {
    let d = standing(Role::Monster, 6.0, -1.0);
    let tc = torso_center(&d);
    assert!(is_live_threat([tc[0] - 1.0, tc[1]], [6.0, 0.0], &d));
}

#[test]
fn slow_away_far_or_off_target_blades_are_not_threats() {
    let d = standing(Role::Monster, 6.0, -1.0);
    let tc = torso_center(&d);
    assert!(!is_live_threat([tc[0] - 1.0, tc[1]], [1.5, 0.0], &d), "below 2.5 m/s");
    assert!(!is_live_threat([tc[0] - 1.0, tc[1]], [-6.0, 0.0], &d), "heading away");
    assert!(!is_live_threat([tc[0] - 4.0, tc[1]], [6.0, 0.0], &d), "contact later than 0.4 s");
    assert!(!is_live_threat([tc[0] - 1.0, tc[1] + 3.0], [6.0, 0.0], &d), "passes 3 m above the body");
}
