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
