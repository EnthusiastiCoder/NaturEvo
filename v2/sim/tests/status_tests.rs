// WS1 status rules. apply_status(role, status, duration, magnitude, source); Knock magnitude = distance in tiles.
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::fighter::{Action, CombatState as S, Role};
use arena_sim::matchup::{Match, Outcome};
use arena_sim::status::Status;
use common::*;

const H: Role = Role::Hero;
const M: Role = Role::Monster;
fn fx(m: &Match, r: Role, s: Status) -> u32 { m.fx(r)[s.idx()] }
fn count(j: &str, pat: &str) -> usize { j.matches(pat).count() }
fn near(a: f32, b: f32, t: f32) -> bool { (a - b).abs() <= t }

#[test]
fn stagger_refresh_is_max_without_stacking() {
    let mut m = duel(8.0);
    m.apply_status(M, Status::Stagger, 10, 1.0, H);
    m.apply_status(M, Status::Stagger, 6, 1.0, H);
    assert_eq!(fx(&m, M, Status::Stagger), 10);
    m.apply_status(M, Status::Stagger, 20, 1.0, H);
    assert_eq!(fx(&m, M, Status::Stagger), 20);
    assert!(m.stacks(M)[Status::Stagger.idx()] <= 1);
}

#[test]
fn stagger_has_12_tick_poise_after_it_ends() {
    let mut m = duel(8.0);
    m.apply_status(M, Status::Stagger, 10, 1.0, H);
    for _ in 0..30 { if fx(&m, M, Status::Stagger) == 0 { break; } m.step_with(idle(), idle()); }
    assert_eq!(fx(&m, M, Status::Stagger), 0);
    m.apply_status(M, Status::Stagger, 10, 1.0, H);
    assert_eq!(fx(&m, M, Status::Stagger), 0, "immune right after");
    run(&mut m, 13, idle(), idle());
    m.apply_status(M, Status::Stagger, 10, 1.0, H);
    assert_eq!(fx(&m, M, Status::Stagger), 10);
}

#[test]
fn stun_diminishing_returns_20_10_6_with_immunity_45() {
    let mut m = duel(8.0);
    let mut got = vec![];
    for _ in 0..3 {
        m.apply_status(M, Status::Stun, 20, 1.0, H);
        got.push(fx(&m, M, Status::Stun));
        for _ in 0..40 { if fx(&m, M, Status::Stun) == 0 { break; } m.step_with(idle(), idle()); }
        m.apply_status(M, Status::Stun, 20, 1.0, H);
        assert_eq!(fx(&m, M, Status::Stun), 0, "immune after stun");
        run(&mut m, 46, idle(), idle());
    }
    assert_eq!(got, vec![20, 10, 6]);
}

#[test]
fn warrior_club_cannot_chain_stuns() {
    let mut m = duel_sp(Species::Warrior, Rank::Grass, 1.0);
    m.set_hp(H, 10000.0);
    let (mut starts, mut prev) = (vec![], 0u32);
    for t in 0..900u32 {
        m.step_with(idle(), Action { fwd: 1.0, ..atk(0) });
        let s = fx(&m, H, Status::Stun);
        if s > 0 && prev == 0 { starts.push(t); }
        prev = s;
    }
    assert!(starts.len() >= 2, "club should stun repeatedly: {starts:?}");
    for w in starts.windows(2) { assert!(w[1] - w[0] >= 51, "stuns too close: {starts:?}"); }
}
