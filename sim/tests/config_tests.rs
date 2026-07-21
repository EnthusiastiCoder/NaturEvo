use jungle_sim::config::{knobs, Config};
use jungle_sim::report::batch;
use jungle_sim::sim::{simulate, simulate_with};

#[test]
fn default_config_values() {
    let c = Config::default();
    assert_eq!(c.monster_hp_mult, 1.0);
    assert_eq!(c.monster_atk_mult, 1.0);
    assert_eq!(c.boss_hp_mult, 1.0);
    assert_eq!(c.boss_atk_mult, 1.0);
    assert_eq!(c.hero_hp_base, 50);
    assert_eq!(c.hero_hp_per_level, 14);
    assert_eq!(c.hero_atk_per_level, 3.0);
    assert_eq!(c.hero_def_per_level, 1.5);
    assert_eq!(c.xp_mult, 1.0);
    assert_eq!(c.gold_mult, 1.0);
    assert_eq!(c.potion_cost, 25);
    assert_eq!(c.boss_level, 32);
    assert_eq!(c.boss_attempts, 8);
    assert_eq!(c.event_cap, 65536);
    assert_eq!(c.postgame_level_cap, 80);
    assert_eq!(c.evolve_kills, 300);
}

#[test]
fn default_matches_baseline() {
    // constants captured from the pre-config build
    // old lengths are now first_win_event + 1; the run itself continues to the event cap
    for (seed, len, lvl) in [(1u64, 10625usize, 40u32), (2, 10543, 40), (42, 10617, 40)] {
        let r = simulate(seed);
        assert_eq!(r.first_win_event.map(|f| f as usize + 1), Some(len), "seed {}", seed);
        assert_eq!(r.events.len(), 65536, "seed {}", seed);
        assert_eq!(r.final_level, lvl, "seed {}", seed);
        let w = simulate_with(seed, &Config::default());
        assert_eq!(w.first_win_event.map(|f| f as usize + 1), Some(len), "seed {}", seed);
        assert_eq!(w.events.len(), 65536, "seed {}", seed);
        assert_eq!(w.final_level, lvl, "seed {}", seed);
    }
    // seed 7 loses on the original build: 10809 events, level 40, no win, no post-game
    let l = simulate(7);
    assert!(!l.won && l.first_win_event.is_none());
    assert_eq!(l.events.len(), 10809);
    assert_eq!(l.final_level, 40);
    assert_eq!(batch(200, 4).wins, 108);
}

#[test]
fn knob_changes_outcome() {
    let wins = |m: f32| {
        let mut c = Config::default();
        c.boss_hp_mult = m;
        (0..100u64).filter(|s| simulate_with(*s, &c).won).count()
    };
    assert_ne!(wins(0.1), wins(5.0));
    let mut c = Config::default();
    c.hero_hp_per_level = 30;
    let a = simulate(1).events[0].hero.max_hp;
    let b = simulate_with(1, &c).events[0].hero.max_hp;
    assert_eq!(a, 64);
    assert_eq!(b, 80);
}

#[test]
fn same_seed_same_cfg_identical() {
    let mut c = Config::default();
    c.monster_atk_mult = 1.7;
    c.xp_mult = 2.0;
    let a = simulate_with(9, &c);
    let b = simulate_with(9, &c);
    assert_eq!(a.events.len(), b.events.len());
    for (x, y) in a.events.iter().zip(b.events.iter()) {
        assert_eq!(x.text, y.text);
        assert_eq!(x.hero.hp, y.hero.hp);
    }
    assert_eq!(a.won, b.won);
}

#[test]
fn set_rejects_unknown_key() {
    let mut c = Config::default();
    let e = c.set("bogus", "1").unwrap_err();
    assert!(e.contains("bogus"), "{}", e);
}

#[test]
fn set_rejects_out_of_range() {
    let mut c = Config::default();
    for (k, v) in [
        ("monster_hp_mult", "0.01"),
        ("monster_hp_mult", "5.1"),
        ("boss_attempts", "0"),
        ("boss_attempts", "51"),
        ("potion_cost", "501"),
        ("xp_mult", "10.5"),
        ("boss_level", "4"),
    ] {
        let e = c.set(k, v).unwrap_err();
        assert!(e.contains(k), "{}", e);
    }
    assert_eq!(c, Config::default());
}

#[test]
fn set_rejects_nan() {
    let mut c = Config::default();
    for v in ["NaN", "nan", "inf", "-inf", "infinity"] {
        let e = c.set("gold_mult", v).unwrap_err();
        assert!(e.contains("gold_mult"), "{}", e);
    }
    assert_eq!(c, Config::default());
}

#[test]
fn set_rejects_garbage() {
    let mut c = Config::default();
    assert!(c.set("boss_level", "abc").unwrap_err().contains("boss_level"));
    assert!(c.set("boss_level", "").is_err());
    assert!(c.set("hero_hp_base", "12.5").is_err());
    assert!(c.set("potion_cost", "-3").is_err());
}

#[test]
fn set_accepts_bounds() {
    let mut c = Config::default();
    c.set("monster_hp_mult", "0.1").unwrap();
    assert_eq!(c.monster_hp_mult, 0.1f32);
    c.set("monster_hp_mult", "5.0").unwrap();
    assert_eq!(c.monster_hp_mult, 5.0);
    c.set("xp_mult", "10").unwrap();
    assert_eq!(c.xp_mult, 10.0);
    c.set("hero_hp_base", "10").unwrap();
    assert_eq!(c.hero_hp_base, 10);
    c.set("hero_hp_base", "500").unwrap();
    assert_eq!(c.hero_hp_base, 500);
    c.set("boss_attempts", "50").unwrap();
    assert_eq!(c.boss_attempts, 50);
    c.set("hero_def_per_level", "0").unwrap();
    assert_eq!(c.hero_def_per_level, 0.0);
    c.set("boss_level", "40").unwrap();
    assert_eq!(c.boss_level, 40);
    assert!(c.validate().is_ok());
}

#[test]
fn knobs_cover_all_fields() {
    let keys: Vec<&str> = knobs().iter().map(|k| k.key).collect();
    let expect = [
        "monster_hp_mult", "monster_atk_mult", "boss_hp_mult", "boss_atk_mult", "hero_hp_base",
        "hero_hp_per_level", "hero_atk_per_level", "hero_def_per_level", "xp_mult", "gold_mult",
        "potion_cost", "boss_level", "boss_attempts", "event_cap", "postgame_level_cap", "evolve_kills",
    "map_size", "encounter_rate", "wander_radius", "elites_per_zone", "elite_mult", "max_steps",
    ];
    assert_eq!(keys.len(), expect.len());
    for k in expect {
        assert!(keys.contains(&k), "missing {}", k);
    }
    // each default, applied via set, reproduces Config::default()
    let mut c = Config::default();
    for k in knobs() {
        assert!(k.min <= k.default && k.default <= k.max, "{}", k.key);
        assert!(!k.label.is_empty());
        c.set(k.key, &format!("{}", k.default)).unwrap();
    }
    assert_eq!(c, Config::default());
}

// ---- map/walk knobs (plan 2026-10-21_hero-map-walking, workstream 1) ----
const NEW_KNOBS: [(&str, f64, f64, f64); 6] = [
    ("map_size", 32.0, 128.0, 64.0),
    ("encounter_rate", 0.01, 0.5, 0.12),
    ("wander_radius", 2.0, 20.0, 6.0),
    ("elites_per_zone", 0.0, 3.0, 1.0),
    ("elite_mult", 1.0, 3.0, 1.4),
    ("max_steps", 1000.0, 50_000_000.0, 5_000_000.0),
];
}
