// Uses assumed hooks: Match::step_with, place_for_duel(dist), face_away(Role).
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::sense::{observe, N_IN};
use arena_sim::world::Arena;

fn mk(seed: u64) -> Match {
    let cfg = ArenaConfig::default();
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    Match::new(Arena::generate(seed, &cfg), h, m, seed, &cfg)
}
fn idle() -> Action { Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false } }

#[test]
fn n_in_is_40() {
    assert_eq!(N_IN, 40);
}

#[test]
fn observation_within_unit_range_for_both_roles() {
    let mut m = mk(1);
    for _ in 0..200 {
        m.step();
        for r in [Role::Hero, Role::Monster] {
            let o = observe(&m, r);
            assert_eq!(o.len(), 40);
            for v in o {
                assert!(v.is_finite() && (-1.0..=1.0).contains(&v), "{v}");
            }
        }
    }
}

#[test]
fn observation_deterministic() {
    assert_eq!(observe(&mk(3), Role::Hero), observe(&mk(3), Role::Hero));
}

#[test]
fn hero_and_monster_views_differ() {
    let m = mk(3);
    assert_ne!(observe(&m, Role::Hero), observe(&m, Role::Monster));
}

#[test]
fn enemy_distance_changes_observation() {
    let mut a = mk(4);
    let mut b = mk(4);
    a.place_for_duel(1.0);
    b.place_for_duel(10.0);
    assert_ne!(observe(&a, Role::Hero), observe(&b, Role::Hero));
}
