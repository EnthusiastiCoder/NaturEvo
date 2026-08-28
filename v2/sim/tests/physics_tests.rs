// Assumed helper beyond the contract: Match::step_with(hero: Action, monster: Action)
// (scripted actions, bypasses nets). Match::new takes cfg by reference.
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn mk(seed: u64) -> (Match, ArenaConfig) {
    let cfg = ArenaConfig::default();
    let arena = Arena::generate(seed, &cfg);
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    (Match::new(arena, h, m, seed, &cfg), cfg)
}
fn idle() -> Action { Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false } }

#[test]
fn rng_is_seeded_and_in_range() {
    let mut a = Rng::new(7);
    let mut b = Rng::new(7);
    for _ in 0..100 { assert_eq!(a.next_u64(), b.next_u64()); }
    let mut c = Rng::new(8);
    assert_ne!(Rng::new(7).next_u64(), c.next_u64());
    let mut r = Rng::new(1);
    for _ in 0..1000 {
        let f = r.f32();
        assert!((0.0..1.0).contains(&f));
        let x = r.range(-2.0, 3.0);
        assert!((-2.0..3.0).contains(&x));
    }
}

#[test]
fn config_defaults() {
    let c = ArenaConfig::default();
    assert_eq!(c.tick_hz, 30);
    assert_eq!(c.arena_w, 48.0);
    assert_eq!(c.arena_h, 32.0);
    assert_eq!(c.fighter_radius, 0.4);
    assert_eq!(c.max_speed, 4.0);
    assert_eq!(c.max_turn, 6.0);
    assert_eq!(c.max_ticks, 1800);
    assert_eq!(c.n_rays, 9);
    assert_eq!(c.vision_fov, 2.094);
    assert_eq!(c.vision_range, 12.0);
    assert_eq!(c.max_stamina, 100.0);
    assert_eq!(c.max_hp, 100.0);
}

#[test]
fn arena_spawns_open_reachable_and_inside() {
    let cfg = ArenaConfig::default();
    for seed in 0..20u64 {
        let a = Arena::generate(seed, &cfg);
        let s = a.spawns();
        for (x, y) in s {
            assert!(x > 0.0 && x < cfg.arena_w && y > 0.0 && y < cfg.arena_h);
            assert!(!a.is_blocked(x, y));
        }
        assert!(a.reachable(s[0], s[1]), "seed {seed}");
        assert!(a.is_blocked(-1.0, 5.0));
        assert!(a.is_blocked(5.0, cfg.arena_h + 1.0));
    }
}

#[test]
fn arena_generation_deterministic_per_seed() {
    let cfg = ArenaConfig::default();
    let a = Arena::generate(5, &cfg);
    let b = Arena::generate(5, &cfg);
    assert_eq!(a.spawns(), b.spawns());
    for y in 0..32 {
        for x in 0..48 {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            assert_eq!(a.is_blocked(fx, fy), b.is_blocked(fx, fy));
        }
    }
}

#[test]
fn full_forward_speed_is_capped_at_max_speed() {
    let (mut m, cfg) = mk(3);
    let p0 = m.pos(Role::Hero);
    m.step_with(Action { fwd: 1.0, ..idle() }, idle());
    let p1 = m.pos(Role::Hero);
    let d = ((p1.0 - p0.0).powi(2) + (p1.1 - p0.1).powi(2)).sqrt();
    let cap = cfg.max_speed / cfg.tick_hz as f32;
    assert!(d <= cap + 1e-4, "moved {d} > {cap}");
    assert!(d > 0.0);
}

#[test]
fn diagonal_move_not_faster_than_forward() {
    let (mut m, cfg) = mk(3);
    let p0 = m.pos(Role::Hero);
    m.step_with(Action { fwd: 1.0, strafe: 1.0, ..idle() }, idle());
    let p1 = m.pos(Role::Hero);
    let d = ((p1.0 - p0.0).powi(2) + (p1.1 - p0.1).powi(2)).sqrt();
    assert!(d <= cfg.max_speed / cfg.tick_hz as f32 + 1e-4);
}

#[test]
fn fighters_never_leave_arena() {
    let (mut m, cfg) = mk(11);
    for i in 0..600 {
        let a = Action { fwd: 1.0, turn: if i % 50 < 10 { 1.0 } else { 0.0 }, ..idle() };
        m.step_with(a, a);
        for r in [Role::Hero, Role::Monster] {
            let (x, y) = m.pos(r);
            assert!(x >= 0.0 && x <= cfg.arena_w && y >= 0.0 && y <= cfg.arena_h);
            assert!(!m.arena().is_blocked(x, y));
        }
    }
}
