// WS2 sensor contract (plan 2026-10-23_v2-combat-depth.md). Hooks: Match::apply_status, set_hp, face_away.
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::Match;
use arena_sim::rng::Rng;
use arena_sim::sense::{idx, observe, N_IN};
use arena_sim::status::Status;
use arena_sim::world::Arena;
use common::*;

fn mk(seed: u64) -> Match {
    let cfg = ArenaConfig::default();
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    Match::new(Arena::generate(seed, &cfg), h, m, seed, &cfg)
}

#[test]
fn n_in_is_80() {
    assert_eq!(N_IN, 80);
}

#[test]
fn idx_constants_match_contract() {
    assert_eq!(idx::HP, 0);
    assert_eq!(idx::STAMINA, 1);
    assert_eq!(idx::DODGE_CD, 2);
    assert_eq!(idx::OWN_PHASE, 3);
    assert_eq!(idx::OWN_STATE, 4);
    assert_eq!(idx::OWN_FX, 16);
    assert_eq!(idx::OWN_ATK, 23);
    assert_eq!(idx::OWN_SPECIES, 26);
    assert_eq!(idx::ENEMY_DIST, 31);
    assert_eq!(idx::ENEMY_SIN, 32);
    assert_eq!(idx::ENEMY_COS, 33);
    assert_eq!(idx::ENEMY_HP, 34);
    assert_eq!(idx::ENEMY_STAMINA, 35);
    assert_eq!(idx::ENEMY_STATE, 36);
    assert_eq!(idx::ENEMY_PHASE, 48);
    assert_eq!(idx::ENEMY_FACE_SIN, 49);
    assert_eq!(idx::ENEMY_FACE_COS, 50);
    assert_eq!(idx::ENEMY_FX, 51);
    assert_eq!(idx::ENEMY_ATK, 58);
    assert_eq!(idx::ENEMY_SPECIES, 61);
    assert_eq!(idx::SEEN, 66);
    assert_eq!(idx::IN_RANGE, 67);
    assert_eq!(idx::BIAS, 68);
    assert_eq!(idx::OWN_X, 69);
    assert_eq!(idx::OWN_Y, 70);
    assert_eq!(idx::RAYS, 71);
}

#[test]
fn observation_within_unit_range_under_fuzz() {
    for seed in 1..=3u64 {
        let mut m = mk(seed);
        let mut rng = Rng::new(seed + 100);
        let mut ra = |r: &mut Rng| Action {
            fwd: r.range(-1.0, 1.0),
            strafe: r.range(-1.0, 1.0),
            turn: r.range(-1.0, 1.0),
            attack: r.f32() < 0.4,
            block: r.f32() < 0.2,
            dodge: r.f32() < 0.05,
            attack_kind: (r.f32() * 3.0) as u8 % 3,
        };
        m.place_for_duel(4.0);
        for t in 0..300 {
            if t % 20 == 0 {
                let s = Status::ALL[(t / 20) % 7];
                m.apply_status(Role::Hero, s, 40, 1.0, Role::Monster);
                m.apply_status(Role::Monster, s, 40, 1.0, Role::Hero);
            }
            let (a, b) = (ra(&mut rng), ra(&mut rng));
            m.step_with(a, b);
            for r in [Role::Hero, Role::Monster] {
                let o = observe(&m, r);
                assert_eq!(o.len(), 80);
                for (i, v) in o.iter().enumerate() {
                    assert!(v.is_finite() && (-1.0..=1.0).contains(v), "idx {i} = {v}");
                }
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
    let (a, b) = (duel(1.0), duel(10.0));
    assert_ne!(observe(&a, Role::Hero), observe(&b, Role::Hero));
}

#[test]
fn facing_away_changes_observation() {
    let a = duel(5.0);
    let mut b = duel(5.0);
    b.face_away(Role::Hero);
    assert_ne!(observe(&a, Role::Hero), observe(&b, Role::Hero));
}

#[test]
fn bias_is_always_one() {
    let m = duel(5.0);
    assert_eq!(observe(&m, Role::Hero)[idx::BIAS], 1.0);
    assert_eq!(observe(&mk(2), Role::Monster)[idx::BIAS], 1.0);
}

#[test]
fn enemy_block_zero_when_unseen_including_status_and_attack() {
    let mut m = duel_sp(Species::Warrior, Rank::Grass, 13.0); // 13 tiles > vision range 12
    m.apply_status(Role::Monster, Status::Confused, 60, 1.0, Role::Hero);
    m.apply_status(Role::Monster, Status::Slowed, 60, 1.0, Role::Hero);
    m.step_with(idle(), atk(1)); // monster in Windup with a latched kind
    let o = observe(&m, Role::Hero);
    assert_eq!(o[idx::SEEN], 0.0);
    for i in idx::ENEMY_DIST..=idx::IN_RANGE {
        assert_eq!(o[i], 0.0, "enemy block index {i} must be zero when unseen");
    }
    assert_eq!(o[idx::BIAS], 1.0);
}

#[test]
fn seen_enemy_windup_shows_kind_phase_and_species() {
    let mut m = duel_sp(Species::Warrior, Rank::Grass, 5.0);
    m.step_with(idle(), atk(1));
    let o = observe(&m, Role::Hero);
    assert_eq!(o[idx::SEEN], 1.0);
    assert_eq!(o[idx::ENEMY_STATE + 1], 1.0, "Windup is state 1");
    let others: f32 = (0..12).filter(|i| *i != 1).map(|i| o[idx::ENEMY_STATE + i]).sum();
    assert_eq!(others, 0.0);
    assert_eq!(o[idx::ENEMY_ATK + 1], 1.0);
    assert_eq!(o[idx::ENEMY_ATK], 0.0);
    assert_eq!(o[idx::ENEMY_ATK + 2], 0.0);
    assert_eq!(o[idx::ENEMY_SPECIES + 3], 1.0, "Warrior is species 3");
    assert_eq!(o[idx::ENEMY_SPECIES], 0.0);
    let p0 = o[idx::ENEMY_PHASE];
    assert!(p0 > 0.0 && p0 <= 1.0, "phase progress {p0}");
    m.step_with(idle(), atk(1));
    m.step_with(idle(), atk(1));
    let p1 = observe(&m, Role::Hero)[idx::ENEMY_PHASE];
    assert_ne!(p0, p1, "phase progress must advance with the windup");
}

#[test]
fn own_state_attack_and_species_one_hot() {
    let mut m = duel(5.0);
    m.step_with(atk(1), idle());
    let o = observe(&m, Role::Hero);
    assert_eq!(o[idx::OWN_STATE + 1], 1.0);
    assert_eq!(o[idx::OWN_ATK + 1], 1.0);
    assert_eq!(o[idx::OWN_ATK], 0.0);
    assert_eq!(o[idx::OWN_SPECIES], 1.0, "hero species is 0");
    let sp: f32 = (1..5).map(|i| o[idx::OWN_SPECIES + i]).sum();
    assert_eq!(sp, 0.0);
    let p = o[idx::OWN_PHASE];
    assert!(p > 0.0 && p <= 1.0);
    let om = observe(&duel_sp(Species::Mage, Rank::Grass, 5.0), Role::Monster);
    assert_eq!(om[idx::OWN_SPECIES + 4], 1.0, "Mage is species 4");
    assert_eq!(om[idx::OWN_SPECIES], 0.0);
}

#[test]
fn stunned_scenario_sets_state_and_status() {
    let mut m = duel(5.0);
    m.apply_status(Role::Hero, Status::Stun, 30, 0.0, Role::Monster);
    m.step_with(idle(), idle());
    let o = observe(&m, Role::Hero);
    assert_eq!(o[idx::OWN_STATE + 10], 1.0, "Stunned is state 10");
    assert!(o[idx::OWN_FX + 2] > 0.0 && o[idx::OWN_FX + 2] <= 1.0);
    let e = observe(&m, Role::Monster);
    assert_eq!(e[idx::ENEMY_STATE + 10], 1.0);
    assert!(e[idx::ENEMY_FX + 2] > 0.0);
}

#[test]
fn status_fractions_track_remaining_ticks_and_mirror_to_enemy() {
    let mut m = duel(5.0);
    m.apply_status(Role::Hero, Status::Confused, 90, 1.0, Role::Monster);
    m.apply_status(Role::Hero, Status::Slowed, 60, 1.0, Role::Monster);
    let mut conf = vec![];
    for _ in 0..3 {
        let o = observe(&m, Role::Hero);
        let e = observe(&m, Role::Monster);
        let (c, s) = (o[idx::OWN_FX + 3], o[idx::OWN_FX + 4]);
        assert!(c > 0.0 && c <= 1.0 && s > 0.0 && s <= 1.0);
        assert_eq!(e[idx::ENEMY_FX + 3], c, "enemy view mirrors own view");
        assert_eq!(e[idx::ENEMY_FX + 4], s);
        conf.push(c);
        for _ in 0..15 {
            m.step_with(idle(), idle());
        }
    }
    assert!(conf[0] > conf[1] && conf[1] > conf[2], "fractions decrease: {conf:?}");
    assert!(((conf[0] - conf[1]) - (conf[1] - conf[2])).abs() < 1e-4, "linear in remaining ticks: {conf:?}");
    // unaffected statuses stay zero
    assert_eq!(observe(&m, Role::Hero)[idx::OWN_FX + 6], 0.0);
}

#[test]
fn in_range_flag_follows_distance() {
    assert!(observe(&duel(1.0), Role::Hero)[idx::IN_RANGE] > 0.0);
    assert_eq!(observe(&duel(8.0), Role::Hero)[idx::IN_RANGE], 0.0);
}
