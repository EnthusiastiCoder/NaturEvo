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
