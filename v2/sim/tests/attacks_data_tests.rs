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

#[test]
fn species_have_two_distinct_moves_with_signature_values() {
    for sp in SPECIES {
        let v = moves(sp, Rank::Grass);
        assert!(v.len() >= 2, "{:?}", sp);
        assert_ne!(v[0], v[1]);
        assert_ne!(v[0].name, v[1].name);
    }
    let w = moves(Species::Warrior, Rank::Grass);
    assert_eq!(w[0].windup, 14);
    assert_eq!(eff(&w[0], Status::Knock).1, 2.5);
    assert_eq!(eff(&w[0], Status::Stun).0, 24);
    assert!((w[1].half_arc - 70.0f32.to_radians()).abs() < 1e-3, "cleave arc 140");
    let h = moves(Species::Mage, Rank::Grass);
    assert_eq!((h[0].windup, h[0].damage), (10, 5.0));
    assert!((h[0].range - 2.0).abs() < 1e-5);
    assert_eq!(eff(&h[0], Status::Poison).0, 240);
    assert_eq!(eff(&h[0], Status::Confused).0, 90);
    assert_eq!(moves(Species::Rogue, Rank::Grass)[0].windup, 3);
}

#[test]
fn elites_are_stronger_with_one_extra_effect() {
    let extra = [(Species::Minion, Status::Slowed), (Species::Rogue, Status::Poison), (Species::Warrior, Status::Bleed), (Species::Mage, Status::Knock)];
    for (sp, st) in extra {
        let (g, e) = (moves(sp, Rank::Grass), moves(sp, Rank::Elite));
        assert_eq!(g.len(), e.len());
        for (a, b) in g.iter().zip(e.iter()) {
            assert!((b.damage - a.damage * 1.15).abs() < 1e-3, "{} dmg", a.name);
            assert_eq!(b.effects.len(), a.effects.len() + 1, "{}", a.name);
        }
        assert!(e.iter().any(|m| has(m, st)), "{:?} elite lacks {:?}", sp, st);
        assert_ne!(g, e);
    }
}

#[test]
fn boss_has_three_moves_including_slam() {
    for sp in [Species::Warrior, Species::Mage] {
        let v = moves(sp, Rank::Boss);
        assert_eq!(v.len(), 3);
        let s = v.iter().find(|m| m.name == "Slam").expect("Slam");
        assert_eq!((s.windup, s.active, s.recovery), (16, 4, 16));
        assert_eq!(eff(s, Status::Stun).0, 30);
        assert_eq!(eff(s, Status::Knock).1, 2.0);
        assert!((20.0..=24.0).contains(&s.damage), "slam dmg {}", s.damage);
        let g = moves(sp, Rank::Grass);
        assert!((v[0].damage - g[0].damage * 1.2).abs() < 1e-3);
    }
}

#[test]
fn all_seven_statuses_appear_in_some_move() {
    let mut all = vec![];
    for sp in [Species::Hero, Species::Minion, Species::Rogue, Species::Warrior, Species::Mage] {
        for rk in [Rank::Grass, Rank::Elite, Rank::Boss] { all.extend(moves(sp, rk)); }
    }
    for s in Status::ALL { assert!(all.iter().any(|m| has(m, s)), "{:?} unused", s); }
}
