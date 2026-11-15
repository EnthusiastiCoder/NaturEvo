use jungle_sim::config::{knobs, Config};
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
    // constants captured with permadeath off (the respawn loop), after the spacing and permadeath
    // changes moved the map and RNG stream; the run continues to the event cap after a win
    let mut off = Config::default();
    off.set("permadeath", "false").unwrap();
    for (seed, len, lvl) in [(7u64, 15437usize, 40u32), (0, 15620, 40)] {
        let r = simulate_with(seed, &off);
        assert_eq!(r.first_win_event.map(|f| f as usize + 1), Some(len), "seed {}", seed);
        assert_eq!(r.events.len(), 65536, "seed {}", seed);
        assert_eq!(r.final_level, lvl, "seed {}", seed);
    }
    // seeds 1 and 4 lose with permadeath off: level 40, no win, no post-game
    for (seed, len) in [(1u64, 15899usize), (4, 15637)] {
        let l = simulate_with(seed, &off);
        assert!(!l.won && l.first_win_event.is_none(), "seed {}", seed);
        assert_eq!(l.events.len(), len, "seed {}", seed);
        assert_eq!(l.final_level, 40, "seed {}", seed);
    }
    assert_eq!(jungle_sim::report::batch_with(200, 4, &off).wins, 81);
    // the default config is permadeath on, so simulate() differs from the off runs
    assert!(simulate(7).outcome == "won" || simulate(7).outcome == "died");
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
        "permadeath", "zone_margin",
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

#[test]
fn map_knobs_present_with_ranges_and_defaults() {
    for (key, min, max, def) in NEW_KNOBS {
        let k = knobs().iter().find(|k| k.key == key).unwrap_or_else(|| panic!("no knob {}", key));
        assert_eq!(k.min, min, "{} min", key);
        assert_eq!(k.max, max, "{} max", key);
        assert!((k.default - def).abs() < 1e-6, "{} default {}", key, k.default);
        assert!(!k.label.is_empty());
    }
    assert_eq!(knobs().len(), 24);
    let c = Config::default();
    assert_eq!(c.map_size as f64, 64.0);
    assert!((c.encounter_rate as f64 - 0.12).abs() < 1e-6);
    assert_eq!(c.wander_radius as f64, 6.0);
    assert_eq!(c.elites_per_zone as f64, 1.0);
    assert!((c.elite_mult as f64 - 1.4).abs() < 1e-6);
    assert_eq!(c.max_steps as f64, 5_000_000.0);
}

#[test]
fn map_knobs_set_rejects_out_of_range_and_garbage() {
    let cases = [
        ("map_size", "31", "129"),
        ("encounter_rate", "0.005", "0.51"),
        ("wander_radius", "1", "21"),
        ("elites_per_zone", "-1", "4"),
        ("elite_mult", "0.9", "3.1"),
        ("max_steps", "999", "50000001"),
    ];
    for (key, below, above) in cases {
        let mut c = Config::default();
        for v in [below, above, "abc", "", "NaN", "inf"] {
            let e = c.set(key, v).unwrap_err();
            assert!(e.contains(key), "{} {:?}: {}", key, v, e);
        }
        assert_eq!(c, Config::default(), "{} must not change on error", key);
    }
    for (key, min, max, _) in NEW_KNOBS {
        let mut c = Config::default();
        c.set(key, &format!("{}", min)).unwrap();
        c.set(key, &format!("{}", max)).unwrap();
        assert!(c.validate().is_ok(), "{}", key);
    }
}

#[test]
fn map_size_and_integer_knobs_reject_fractions() {
    let mut c = Config::default();
    for key in ["map_size", "wander_radius", "elites_per_zone", "max_steps"] {
        assert!(c.set(key, "10.5").is_err(), "{} must be an integer", key);
    }
    assert_eq!(c, Config::default());
    c.set("map_size", "48").unwrap();
    assert_eq!(c.map_size as f64, 48.0);
    c.set("encounter_rate", "0.25").unwrap();
    assert!((c.encounter_rate as f64 - 0.25).abs() < 1e-6);
}

#[test]
fn map_knobs_validate_rejects_out_of_range_fields() {
    let mut c = Config::default();
    c.map_size = 31;
    assert!(c.validate().unwrap_err().contains("map_size"));
    let mut c = Config::default();
    c.map_size = 129;
    assert!(c.validate().is_err());
    let mut c = Config::default();
    c.encounter_rate = 0.0;
    assert!(c.validate().unwrap_err().contains("encounter_rate"));
    let mut c = Config::default();
    c.encounter_rate = 0.51;
    assert!(c.validate().is_err());
    let mut c = Config::default();
    c.wander_radius = 1;
    assert!(c.validate().unwrap_err().contains("wander_radius"));
    let mut c = Config::default();
    c.elites_per_zone = 4;
    assert!(c.validate().unwrap_err().contains("elites_per_zone"));
    let mut c = Config::default();
    c.elite_mult = 0.9;
    assert!(c.validate().unwrap_err().contains("elite_mult"));
    let mut c = Config::default();
    c.max_steps = 999;
    assert!(c.validate().unwrap_err().contains("max_steps"));
    let mut c = Config::default();
    c.max_steps = 50_000_001;
    assert!(c.validate().is_err());
}

#[test]
fn knob_keys_match_new_set() {
    let keys: Vec<&str> = knobs().iter().map(|k| k.key).collect();
    for (k, _, _, _) in NEW_KNOBS {
        assert!(keys.contains(&k), "missing {}", k);
    }
    let mut c = Config::default();
    for k in knobs() {
        c.set(k.key, &format!("{}", k.default)).unwrap_or_else(|e| panic!("{}: {}", k.key, e));
    }
    assert!(c.validate().is_ok());
}

// ---- Addendum 4: permadeath toggle (written before implementation) ----
// Picked API: Config.permadeath: bool, Knob.kind: &'static str ("number" or "bool").

#[test]
fn permadeath_defaults_to_true() {
    assert!(Config::default().permadeath);
}

#[test]
fn permadeath_set_accepts_true_false_1_0() {
    let mut c = Config::default();
    for (v, want) in [("false", false), ("true", true), ("0", false), ("1", true)] {
        c.set("permadeath", v).unwrap();
        assert_eq!(c.permadeath, want, "{}", v);
        assert!(c.validate().is_ok());
    }
}

#[test]
fn permadeath_set_rejects_junk_and_keeps_value() {
    let mut c = Config::default();
    for v in ["2", "abc", "", "yes", "-1", "0.5"] {
        let e = c.set("permadeath", v).unwrap_err();
        assert!(e.contains("permadeath"), "{}: {}", v, e);
        assert!(c.permadeath, "value changed by rejected input {:?}", v);
    }
}

#[test]
fn permadeath_knob_is_boolean_and_others_numeric() {
    let k = knobs().iter().find(|k| k.key == "permadeath").expect("permadeath knob");
    assert_eq!(k.kind, "bool");
    assert_eq!(k.default, 1.0);
    for o in knobs().iter().filter(|k| k.key != "permadeath") {
        assert_eq!(o.kind, "number", "{}", o.key);
    }
}
