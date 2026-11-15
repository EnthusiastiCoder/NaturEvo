// Updated for combat depth: hero Light = wu4 ac3 rc6, dmg 8, cycle 13; guard rules in combat_depth_tests.rs.
mod common;
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
use common::*;

#[test]
fn start_state_full_hp_and_stamina() {
    let m = duel(1.0);
    assert_eq!(m.hp(Role::Hero), 100.0);
    assert_eq!(m.hp(Role::Monster), 100.0);
    assert_eq!(m.stamina(Role::Hero), 100.0);
}

#[test]
fn light_attack_windup_4_damage_8_once() {
    let mut m = duel(1.0);
    m.step_with(atk(0), idle());
    for _ in 0..3 {
        assert_eq!(m.hp(Role::Monster), 100.0, "no damage during windup");
        m.step_with(idle(), idle());
    }
    m.step_with(idle(), idle());
    assert_eq!(m.hp(Role::Monster), 92.0, "hit lands on step 5");
    for _ in 0..12 { m.step_with(idle(), idle()); }
    assert_eq!(m.hp(Role::Monster), 92.0, "damage once per swing");
}

#[test]
fn attack_out_of_range_or_outside_arc_misses() {
    let mut m = duel(2.0);
    for _ in 0..30 { m.step_with(atk(0), idle()); }
    assert_eq!(m.hp(Role::Monster), 100.0);
    let mut m = duel(1.0);
    m.face_away(Role::Hero);
    for _ in 0..30 { m.step_with(atk(0), idle()); }
    assert_eq!(m.hp(Role::Monster), 100.0);
}

#[test]
fn swing_cycle_is_13_ticks_when_hitting() {
    let mut m = duel(1.0);
    for _ in 0..17 { m.step_with(atk(0), idle()); }
    assert_eq!(m.hp(Role::Monster), 92.0);
    m.step_with(atk(0), idle());
    assert_eq!(m.hp(Role::Monster), 84.0, "second hit on step 18");
}

#[test]
fn dodge_costs_20_stamina() {
    let mut m = duel(1.0);
    m.step_with(idle(), dge());
    assert!((m.stamina(Role::Monster) - 80.0).abs() < 1.5);
}

#[test]
fn dodge_iframes_avoid_hit() {
    let mut m = duel(1.0);
    for _ in 0..3 { m.step_with(atk(0), idle()); }
    for _ in 0..10 { m.step_with(atk(0), dge()); }
    assert_eq!(m.hp(Role::Monster), 100.0);
}

#[test]
fn dodge_cooldown_45_ticks_blocks_second_dodge() {
    let mut m = duel(1.0);
    m.step_with(idle(), dge());
    for _ in 0..10 { m.step_with(idle(), idle()); }
    let s = m.stamina(Role::Monster);
    m.step_with(idle(), dge());
    assert!(m.stamina(Role::Monster) >= s, "second dodge inside cooldown must not cost stamina");
}

#[test]
fn stamina_regens_12_per_second_idle_and_caps() {
    let mut m = duel(1.0);
    m.set_stamina(Role::Hero, 10.0);
    for _ in 0..30 { m.step_with(idle(), idle()); }
    assert!((m.stamina(Role::Hero) - 22.0).abs() < 0.2);
    for _ in 0..600 { m.step_with(idle(), idle()); }
    assert_eq!(m.stamina(Role::Hero), 100.0);
}

#[test]
fn death_ends_match_with_hero_win() {
    let mut m = duel(1.0);
    for _ in 0..1800 {
        if m.finished() { break; }
        m.step_with(atk(0), idle());
    }
    assert!(m.finished());
    assert_eq!(m.outcome(), Outcome::HeroWin);
    assert_eq!(m.hp(Role::Monster), 0.0);
    assert!(m.tick() < 1800);
}

#[test]
fn timeout_is_draw() {
    let mut m = duel(8.0);
    for _ in 0..1800 { m.step_with(idle(), idle()); }
    assert!(m.finished());
    assert_eq!(m.outcome(), Outcome::Draw);
}

#[test]
fn play_result_and_log_consistent() {
    let cfg = ArenaConfig::default();
    let mut r = Rng::new(9);
    let h = Genome::random(Role::Hero, &mut r);
    let mo = Genome::random(Role::Monster, &mut r);
    let res = play(Arena::generate(9, &cfg), h, mo, 9, &cfg);
    assert!(res.ticks >= 1 && res.ticks <= 1800);
    assert!(res.hero_hp >= 0.0 && res.hero_hp <= 100.0);
    assert!(res.monster_hp >= 0.0 && res.monster_hp <= 100.0);
    assert!((res.damage_dealt[0] - (100.0 - res.monster_hp)).abs() < 1e-3);
    assert!((res.damage_dealt[1] - (100.0 - res.hero_hp)).abs() < 1e-3);
    let j = res.log.to_json();
    assert!(j.contains("\"frames\""));
    for k in ["pos", "facing", "hp", "stamina", "action", "state"] {
        assert!(j.contains(k), "log missing {k}");
    }
}
