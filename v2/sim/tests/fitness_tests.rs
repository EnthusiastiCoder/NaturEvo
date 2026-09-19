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
