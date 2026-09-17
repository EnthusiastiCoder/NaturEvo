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

#[test]
fn knockback_moves_2_tiles_over_6_ticks() {
    let mut m = duel(8.0);
    let x0 = m.pos(M).0;
    m.apply_status(M, Status::Knock, 6, 2.0, H);
    run(&mut m, 6, idle(), idle());
    assert!(near(m.pos(M).0 - x0, 2.0, 0.05), "moved {}", m.pos(M).0 - x0);
    run(&mut m, 10, idle(), idle());
    assert!(near(m.pos(M).0 - x0, 2.0, 0.05), "stops after 6 ticks");
    assert_eq!(m.fighter(M).state, S::Idle);
}

fn open_edge_x(m: &Match) -> f32 {
    let mut x = 40.0f32;
    while !m.arena().circle_blocked(x + 0.1, 16.5, 0.4) { x += 0.1; }
    x
}

#[test]
fn wall_slam_deals_3_and_staggers_once() {
    let mut m = duel(1.0);
    let ex = open_edge_x(&m);
    m.set_pos(M, ex, 16.5);
    m.set_pos(H, ex - 1.0, 16.5);
    m.set_facing(H, 0.0);
    m.set_facing(M, std::f32::consts::PI);
    run(&mut m, 13, atk(1), idle());
    run(&mut m, 30, idle(), idle());
    assert_eq!(m.hp(M), 100.0 - 18.0 - 3.0);
    assert_eq!(count(&m.log().to_json(), "\"t\":\"slam\""), 1);
}

#[test]
fn blocking_halves_knockback() {
    let mut m = duel(1.0);
    run(&mut m, 10, idle(), blk());
    let x0 = m.pos(M).0;
    run(&mut m, 13, atk(1), blk());
    run(&mut m, 8, idle(), blk());
    assert!(near(m.pos(M).0 - x0, 1.0, 0.1), "blocked knock {}", m.pos(M).0 - x0);
}

#[test]
fn lunge_does_not_pass_through_target() {
    let mut m = duel(1.0);
    run(&mut m, 15, atk(2), idle());
    assert!(dist(&m) >= 0.8 - 1e-3, "dist {}", dist(&m));
}

#[test]
fn confused_rotates_move_vector_60_degrees_flipping_per_window() {
    let heading = |m: &mut Match, n: u32| {
        let p0 = m.pos(H);
        run(m, n, fwd(1.0), idle());
        let p1 = m.pos(H);
        (p1.1 - p0.1).atan2(p1.0 - p0.0).to_degrees()
    };
    let mut m = duel(20.0);
    m.apply_status(H, Status::Confused, 90, 1.0, M);
    let (a, b) = (heading(&mut m, 14), heading(&mut m, 16));
    assert!(near(a.abs(), 60.0, 6.0), "window 1 heading {a}");
    assert!(near(b.abs(), 60.0, 6.0), "window 2 heading {b}");
    assert!(a * b < 0.0, "sign flips: {a} {b}");
}

#[test]
fn confused_inverts_turn_on_alternate_windows_bits_unaffected() {
    let mut m = duel(20.0);
    m.apply_status(H, Status::Confused, 90, 1.0, M);
    let turn = Action { turn: 0.5, ..idle() };
    let f0 = m.fighter(H).facing;
    run(&mut m, 15, turn, idle());
    let f1 = m.fighter(H).facing;
    run(&mut m, 15, turn, idle());
    let f2 = m.fighter(H).facing;
    let (d1, d2) = (f1 - f0, f2 - f1);
    assert!(near(d1.abs(), 1.5, 0.05) && near(d2.abs(), 1.5, 0.05), "{d1} {d2}");
    assert!(d1 * d2 < 0.0);
    let mut m = duel(1.0);
    m.apply_status(H, Status::Confused, 90, 1.0, M);
    run(&mut m, 5, atk(0), idle());
    assert_eq!(m.hp(M), 92.0, "attack bit unaffected");
}

#[test]
fn confusion_is_deterministic() {
    let go = || {
        let mut m = duel(20.0);
        m.apply_status(H, Status::Confused, 90, 1.0, M);
        run(&mut m, 60, fwd(1.0), idle());
        m.pos(H)
    };
    assert_eq!(go(), go());
}

#[test]
fn slowed_stacks_to_two_speed_and_turn() {
    let step_len = |n_apply: u32| {
        let mut m = duel(20.0);
        for _ in 0..n_apply { m.apply_status(H, Status::Slowed, 90, 1.0, M); }
        let p0 = m.pos(H);
        m.step_with(fwd(1.0), idle());
        let p1 = m.pos(H);
        ((p1.0 - p0.0).powi(2) + (p1.1 - p0.1).powi(2)).sqrt()
    };
    assert!(near(step_len(1), 0.08, 1e-3));
    assert!(near(step_len(2), 0.048, 1e-3));
    assert!(near(step_len(3), 0.048, 1e-3), "max 2 stacks");
    let mut m = duel(20.0);
    m.apply_status(H, Status::Slowed, 90, 1.0, M);
    let f0 = m.fighter(H).facing;
    m.step_with(Action { turn: 1.0, ..idle() }, idle());
    assert!(near(m.fighter(H).facing - f0, 0.14, 1e-3));
}

#[test]
fn bleed_ticks_1_per_15_per_stack_up_to_3() {
    let mut m = duel(8.0);
    m.apply_status(H, Status::Bleed, 300, 1.0, M);
    run(&mut m, 31, idle(), idle());
    assert_eq!(m.hp(H), 98.0);
    let mut m = duel(8.0);
    for _ in 0..5 { m.apply_status(H, Status::Bleed, 300, 1.0, M); }
    assert_eq!(m.stacks(H)[Status::Bleed.idx()], 3);
    run(&mut m, 16, idle(), idle());
    assert_eq!(m.hp(H), 97.0);
}

#[test]
fn bleed_doubles_while_moving() {
    let mut m = duel(8.0);
    m.apply_status(H, Status::Bleed, 300, 1.0, M);
    m.set_pos(H, 14.0, 16.5);
    run(&mut m, 31, Action { fwd: 0.6, turn: 0.0, ..idle() }, idle());
    assert_eq!(m.hp(H), 96.0);
}

#[test]
fn bleed_kills_and_credits_source() {
    let mut m = duel(8.0);
    m.set_hp(H, 1.5);
    m.apply_status(H, Status::Bleed, 300, 1.0, M);
    run(&mut m, 31, idle(), idle());
    assert!(m.finished());
    assert_eq!(m.outcome(), Outcome::MonsterWin);
    assert!(m.damage_dealt()[M.index()] >= 1.5 - 1e-3);
    assert!(m.log().to_json().contains("\"t\":\"dot\""));
}

#[test]
fn bleed_refresh_is_max_and_expires() {
    let mut m = duel(8.0);
    m.apply_status(H, Status::Bleed, 30, 1.0, M);
    m.apply_status(H, Status::Bleed, 60, 1.0, M);
    assert_eq!(fx(&m, H, Status::Bleed), 60);
    m.apply_status(H, Status::Bleed, 10, 1.0, M);
    assert_eq!(fx(&m, H, Status::Bleed), 60);
    let mut m = duel(8.0);
    m.apply_status(H, Status::Bleed, 45, 1.0, M);
    run(&mut m, 120, idle(), idle());
    let lost = 100.0 - m.hp(H);
    assert!((2.0..=3.0).contains(&lost), "lost {lost}");
    assert_eq!(fx(&m, H, Status::Bleed), 0);
}

#[test]
fn poison_never_kills_floor_hp_1() {
    let mut m = duel(8.0);
    m.set_hp(H, 2.0);
    m.apply_status(H, Status::Poison, 300, 1.0, M);
    run(&mut m, 200, idle(), idle());
    assert_eq!(m.hp(H), 1.0);
    assert!(!m.finished());
}
