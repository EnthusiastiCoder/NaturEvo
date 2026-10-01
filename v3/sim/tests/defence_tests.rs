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

#[test]
fn every_live_threat_resolves_exactly_once() {
    let mut m = Match::new(11, Brain::Scripted, Brain::Fixed(GUARD_TARGETS));
    let r = m.run_to_end();
    assert!(!m.threats.is_empty(), "scripted attacker must create live threats");
    let mut ids: Vec<u32> = m.threats.iter().map(|t| t.id).collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n, "threat ids unique");
    assert!(m.threats.iter().all(|t| t.resolution.is_some()), "no unresolved threats after the match");
    for who in 0..2 {
        let t = &r.defence[who];
        let count = m.threats.iter().filter(|x| x.defender == who).count() as u32;
        assert_eq!(t.parry + t.block + t.dodge + t.hit + t.miss, count, "tally of fighter {}", who);
    }
}

#[test]
fn scripted_attacker_against_a_held_guard_records_blocked() {
    let mut m = Match::new(11, Brain::Scripted, Brain::Fixed(GUARD_TARGETS));
    let r = m.run_to_end();
    assert!(r.defence[1].block >= 1, "{:?}", r.defence[1]);
    assert!(m.threats.iter().any(|t| t.resolution == Some(Resolution::Blocked)));
}

#[test]
fn standing_dummy_takes_hits_not_blocks() {
    let mut m = Match::new(11, Brain::Scripted, Brain::Dummy);
    let r = m.run_to_end();
    assert!(r.defence[1].hit >= 1);
    assert_eq!(r.defence[1].block + r.defence[1].parry, 0);
}

#[test]
fn no_threats_and_no_credit_when_nobody_swings() {
    let mut m = Match::new(2, Brain::Fixed(GUARD_TARGETS), Brain::Dummy);
    let r = m.run_to_end();
    assert!(m.threats.is_empty());
    assert_eq!(r.defence_credit, [0.0, 0.0]);
}
