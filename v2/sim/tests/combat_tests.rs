// Assumed test hooks beyond the contract (listed in the report):
// Match::step_with(Action, Action), place_for_duel(dist) (face each other, open ground),
// face_away(Role), set_stamina(Role, f32).
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Match, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn idle() -> Action { Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false } }
fn atk() -> Action { Action { attack: true, ..idle() } }
fn blk() -> Action { Action { block: true, ..idle() } }
fn dge() -> Action { Action { dodge: true, ..idle() } }

fn duel(seed: u64) -> Match {
    let cfg = ArenaConfig::default();
    let arena = Arena::generate(seed, &cfg);
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    let mut mt = Match::new(arena, h, m, seed, &cfg);
    mt.place_for_duel(1.0);
    mt
}

#[test]
fn start_state_full_hp_and_stamina() {
    let m = duel(1);
    assert_eq!(m.hp(Role::Hero), 100.0);
    assert_eq!(m.hp(Role::Monster), 100.0);
    assert_eq!(m.stamina(Role::Hero), 100.0);
}

#[test]
fn attack_windup_6_then_active_4_damage_once() {
    let mut m = duel(1);
    m.step_with(atk(), idle());
    for _ in 0..5 {
        assert_eq!(m.hp(Role::Monster), 100.0, "no damage during windup");
        m.step_with(idle(), idle());
    }
    for _ in 0..10 { m.step_with(idle(), idle()); }
    assert_eq!(m.hp(Role::Monster), 88.0, "damage 12 once per swing");
}

#[test]
fn attack_out_of_range_or_outside_arc_misses() {
    let mut m = duel(2);
    m.place_for_duel(2.0);
    for _ in 0..30 { m.step_with(atk(), idle()); }
    assert_eq!(m.hp(Role::Monster), 100.0);
    let mut m = duel(2);
    m.face_away(Role::Hero);
    for _ in 0..30 { m.step_with(atk(), idle()); }
    assert_eq!(m.hp(Role::Monster), 100.0);
}

#[test]
fn swing_cycle_is_18_ticks_no_acting_in_recovery() {
    let mut m = duel(1);
    for _ in 0..36 { m.step_with(atk(), idle()); }
    assert_eq!(m.hp(Role::Monster), 100.0 - 2.0 * 12.0);
}

#[test]
fn block_cuts_damage_80_percent_and_costs_15_stamina() {
    let mut m = duel(1);
    for _ in 0..3 { m.step_with(idle(), blk()); }
    for _ in 0..20 { m.step_with(atk(), blk()); }
    let lost = 100.0 - m.hp(Role::Monster);
    assert!((lost - 2.4).abs() < 1e-3, "12 * 0.2, got {lost}");
    assert!((m.stamina(Role::Monster) - 85.0).abs() < 2.0);
}

#[test]
fn block_does_not_cover_attacks_from_behind() {
    let mut m = duel(1);
    m.face_away(Role::Monster);
    for _ in 0..20 { m.step_with(atk(), blk()); }
    assert_eq!(m.hp(Role::Monster), 88.0);
}

#[test]
fn guard_break_at_zero_stamina_takes_full_damage() {
    let mut m = duel(1);
    m.set_stamina(Role::Monster, 0.0);
    for _ in 0..3 { m.step_with(idle(), blk()); }
    for _ in 0..20 { m.step_with(atk(), blk()); }
    assert_eq!(m.hp(Role::Monster), 88.0);
}

#[test]
fn dodge_costs_30_stamina() {
    let mut m = duel(1);
    m.step_with(idle(), dge());
    assert!((m.stamina(Role::Monster) - 70.0).abs() < 1.5);
}

#[test]
fn dodge_iframes_avoid_hit() {
    let mut m = duel(1);
    for _ in 0..4 { m.step_with(atk(), idle()); }
    for _ in 0..10 { m.step_with(atk(), dge()); }
    assert_eq!(m.hp(Role::Monster), 100.0);
}

#[test]
fn dodge_cooldown_45_ticks_blocks_second_dodge() {
    let mut m = duel(1);
    m.step_with(idle(), dge());
    for _ in 0..10 { m.step_with(idle(), idle()); }
    let s = m.stamina(Role::Monster);
    m.step_with(idle(), dge());
    assert!(m.stamina(Role::Monster) >= s, "second dodge inside cooldown must not cost stamina");
}

#[test]
fn stamina_regens_12_per_second_idle_and_caps() {
    let mut m = duel(1);
    m.set_stamina(Role::Hero, 10.0);
    for _ in 0..30 { m.step_with(idle(), idle()); }
    assert!((m.stamina(Role::Hero) - 22.0).abs() < 0.2);
    for _ in 0..600 { m.step_with(idle(), idle()); }
    assert_eq!(m.stamina(Role::Hero), 100.0);
}

#[test]
fn death_ends_match_with_hero_win() {
    let mut m = duel(1);
    for _ in 0..1800 {
        if m.finished() { break; }
        m.step_with(atk(), idle());
    }
    assert!(m.finished());
    assert_eq!(m.outcome(), Outcome::HeroWin);
    assert_eq!(m.hp(Role::Monster), 0.0);
    assert!(m.tick() < 1800);
}

#[test]
fn timeout_is_draw() {
    let mut m = duel(1);
    m.place_for_duel(8.0);
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
