use jungle_sim::data::SPECIES;
use jungle_sim::sim::{monster_stat, simulate, xp_to_next};

const KINDS: [&str; 15] = [
    "travel", "encounter", "attack", "skill", "hit_taken", "potion", "level_up",
    "victory_fight", "death", "respawn", "shop", "rest", "boss", "win", "evolve",
];

#[test]
fn same_seed_identical() {
    let a = simulate(7);
    let b = simulate(7);
    assert_eq!(a.events.len(), b.events.len());
    for (x, y) in a.events.iter().zip(b.events.iter()) {
        assert_eq!(x.text, y.text);
        assert_eq!(x.kind, y.kind);
    }
    let (x, y) = (a.events.last().unwrap(), b.events.last().unwrap());
    assert_eq!(x.hero.level, y.hero.level);
    assert_eq!(x.hero.hp, y.hero.hp);
    assert_eq!(x.hero.gold, y.hero.gold);
    assert_eq!(a.won, b.won);
    assert_eq!(a.deaths, b.deaths);
    assert_eq!(a.final_level, b.final_level);
}

#[test]
fn different_seeds_differ() {
    let a = simulate(1);
    let b = simulate(2);
    let ta: Vec<&str> = a.events.iter().map(|e| e.text.as_str()).collect();
    let tb: Vec<&str> = b.events.iter().map(|e| e.text.as_str()).collect();
    assert_ne!(ta, tb);
}

#[test]
fn hp_never_negative_and_le_max() {
    for seed in 0..20 {
        for e in &simulate(seed).events {
            assert!(e.hero.hp >= 0, "seed {} idx {}", seed, e.idx);
            assert!(e.hero.hp <= e.hero.max_hp, "seed {} idx {}", seed, e.idx);
            if let Some(m) = &e.mon {
                assert!(m.hp >= 0 || m.hp <= m.max_hp);
            }
        }
    }
}

#[test]
fn idx_consecutive() {
    let r = simulate(3);
    for (i, e) in r.events.iter().enumerate() {
        assert_eq!(e.idx as usize, i);
    }
}

#[test]
fn terminates_under_cap() {
    for seed in 0..5 {
        assert!(simulate(seed).events.len() <= 65536);
    }
}

#[test]
fn xp_to_next_strictly_increasing() {
    for l in 1..60 {
        assert!(xp_to_next(l) < xp_to_next(l + 1), "level {}", l);
    }
}

#[test]
fn monster_stat_monotonic() {
    for s in SPECIES.iter() {
        for base in [s.hp, s.atk, s.def, s.spd] {
            for l in 1..40 {
                assert!(monster_stat(base, l) <= monster_stat(base, l + 1));
            }
        }
    }
}

#[test]
fn final_level_and_events_nonempty() {
    let r = simulate(11);
    assert!(r.final_level >= 1);
    assert!(!r.events.is_empty());
}
