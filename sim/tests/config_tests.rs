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
