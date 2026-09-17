// WS1 combat-depth rules. Tick convention: attack input at step 1 => hit lands on step W+1 (1-indexed).
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, CombatState as S, Role};
use arena_sim::matchup::{Match, BLOCK_HALF_ARC};
use arena_sim::status::Status;
use common::*;
use std::f32::consts::PI;

const H: Role = Role::Hero;
const M: Role = Role::Monster;
fn st(m: &Match, r: Role) -> S { m.fighter(r).state }
fn all_on() -> Action { Action { fwd: 1.0, attack: true, block: true, dodge: true, ..idle() } }
/// Hero swings once at step 1 then feeds `inp`; returns the number of consecutive Recovery frames.
fn recovery_len(m: &mut Match, inp: Action) -> u32 {
    m.step_with(atk(0), idle());
    let mut seen = false;
    let mut n = 0;
    for _ in 0..40 {
        if st(m, H) == S::Recovery { seen = true; n += 1; } else if seen { break; }
        m.step_with(inp, idle());
    }
    n
}

#[test]
fn light_hits_at_step_5_heavy_at_step_13() {
    let mut m = duel(1.0);
    run(&mut m, 4, atk(0), idle());
    assert_eq!(m.hp(M), 100.0);
    m.step_with(idle(), idle());
    assert_eq!(m.hp(M), 92.0);
    let mut m = duel(1.0);
    m.step_with(atk(1), idle());
    run(&mut m, 11, idle(), idle());
    assert_eq!(m.hp(M), 100.0, "heavy still winding up at step 12");
    m.step_with(idle(), idle());
    assert_eq!(m.hp(M), 82.0, "heavy dmg 18 at step 13");
}
