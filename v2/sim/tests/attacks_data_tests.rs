// WS1 move tables and log contract. Plan inconsistency: plan says total frames 14..46, hero Light is 4+3+6 = 13, so bound is 13..=46.
mod common;
use arena_sim::attacks::{moves, Move, Rank, Species};
use arena_sim::status::Status;
use common::*;
use std::f32::consts::PI;

fn has(m: &Move, s: Status) -> bool { m.effects.iter().any(|e| e.status == s) }
fn eff(m: &Move, s: Status) -> (u32, f32) {
    let e = m.effects.iter().find(|e| e.status == s).unwrap_or_else(|| panic!("{} lacks {:?}", m.name, s));
    (e.duration, e.magnitude)
}
const SPECIES: [Species; 4] = [Species::Minion, Species::Rogue, Species::Warrior, Species::Mage];

#[test]
fn hero_has_three_moves_with_table_values() {
    let v = moves(Species::Hero, Rank::Grass);
    assert_eq!(v.len(), 3);
    let t = |i: usize, w, a, r, d: f32, rng: f32, cost: f32| {
        let m = &v[i];
        assert_eq!((m.windup, m.active, m.recovery), (w, a, r), "{}", m.name);
        assert_eq!(m.damage, d);
        assert!((m.range - rng).abs() < 1e-5);
        assert_eq!(m.stamina_cost, cost);
    };
    t(0, 4, 3, 6, 8.0, 1.2, 5.0);
    t(1, 12, 4, 14, 18.0, 1.3, 15.0);
    t(2, 8, 3, 12, 10.0, 1.6, 12.0);
    assert!((v[0].half_arc - PI / 4.0).abs() < 1e-5 && (v[1].half_arc - PI / 4.0).abs() < 1e-5);
    assert!((v[2].half_arc - PI / 6.0).abs() < 1e-5);
    assert_eq!(v[0].lunge, 0.0);
    assert_eq!(v[2].lunge, 2.0);
    assert_eq!(eff(&v[0], Status::Stagger).0, 8);
    assert_eq!(eff(&v[1], Status::Knock).1, 2.0);
    assert_eq!(eff(&v[1], Status::Stun).0, 20);
    assert_eq!(eff(&v[2], Status::Slowed).0, 90);
    assert_eq!(eff(&v[2], Status::Stagger).0, 6);
    assert!(v[0].windup < v[1].windup, "light strictly faster than heavy");
}
