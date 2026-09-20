// WS3 anti-degenerate fitness. Hooks: evolve::{score_result, terms}, Match::{result, apply_status, set_hp, face_away}.
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{score_result, terms};
use arena_sim::fighter::{Action, Role};
use arena_sim::matchup::{Match, MatchResult};
use arena_sim::status::Status;
use common::*;

fn cfg() -> ArenaConfig { ArenaConfig::default() }

/// Step with fixed scripted actions until finished (max ticks).
fn play_out(mut m: Match, a: Action, b: Action) -> MatchResult {
    while !m.finished() { m.step_with(a, b); }
    m.result()
}

fn score(role: Role, r: &MatchResult) -> f32 { score_result(role, r, &cfg()) }

fn fwd_atk(kind: u8) -> Action { Action { fwd: 1.0, ..atk(kind) } }

/// Hero blocks facing away from an idle monster the whole match: a draw.
fn turtle_draw() -> MatchResult {
    let mut m = duel(13.0);
    m.face_away(Role::Hero);
    play_out(m, blk(), idle())
}
/// Hero idles and is beaten by a close aggressive monster; the hero deals no damage.
fn no_damage_loss() -> MatchResult {
    let m = duel(1.0);
    let r = play_out(m, idle(), fwd_atk(0));
    assert_eq!(r.damage_dealt[0], 0.0);
    r
}
/// Both fight at close range.
fn trade() -> MatchResult { play_out(duel(1.0), fwd_atk(0), fwd_atk(0)) }

#[test]
fn fitness_defaults_match_plan() {
    let f = cfg().fitness;
    assert_eq!(f.draw_penalty, 6.0);
    assert!((f.punish_reward - 0.6).abs() < 1e-6);
    assert!((f.status_reward - 0.15).abs() < 1e-6);
    assert_eq!(f.punish_cap as f32, 3.0);
    assert_eq!(f.status_cap as f32, 6.0);
}

#[test]
fn turtle_below_no_damage_loss_below_trader() {
    let (t, l, tr) = (turtle_draw(), no_damage_loss(), trade());
    assert_eq!(t.outcome, arena_sim::matchup::Outcome::Draw);
    assert_eq!(l.outcome, arena_sim::matchup::Outcome::MonsterWin);
    let (st, sl, str_) = (score(Role::Hero, &t), score(Role::Hero, &l), score(Role::Hero, &tr));
    assert!(st < sl, "turtle {st} must be below no-damage loss {sl}");
    assert!(sl < str_, "no-damage loss {sl} must be below trader {str_}");
}

#[test]
fn draw_strictly_worse_than_no_damage_loss() {
    let draw = play_out(duel(13.0), idle(), idle());
    assert_eq!(draw.outcome, arena_sim::matchup::Outcome::Draw);
    assert!(score(Role::Hero, &draw) < score(Role::Hero, &no_damage_loss()));
}
