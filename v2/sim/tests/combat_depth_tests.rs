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

#[test]
fn attacks_cost_stamina() {
    let mut m = duel(1.0);
    m.step_with(atk(0), idle());
    assert!((m.stamina(H) - 95.0).abs() < 1.0, "light cost 5, got {}", m.stamina(H));
    let mut m = duel(1.0);
    m.step_with(atk(1), idle());
    assert!((m.stamina(H) - 85.0).abs() < 1.0, "heavy cost 15, got {}", m.stamina(H));
}

#[test]
fn heavy_refused_below_cost() {
    let mut m = duel(1.0);
    m.set_stamina(H, 5.0);
    for _ in 0..10 {
        m.step_with(atk(1), idle());
        assert_ne!(st(&m, H), S::Windup);
    }
    assert_eq!(m.hp(M), 100.0);
}

#[test]
fn no_acting_in_recovery_and_hit_recovery_is_6() {
    let mut m = duel(1.0);
    assert_eq!(recovery_len(&mut m, all_on()), 6);
    assert_eq!(m.hp(M), 92.0, "only the first swing landed; recovery ignores attack");
}

#[test]
fn whiff_recovery_is_9_vs_hit_6() {
    let mut m = duel(3.0);
    assert_eq!(recovery_len(&mut m, idle()), 9);
    let mut m = duel(1.0);
    assert_eq!(recovery_len(&mut m, idle()), 6);
}

#[test]
fn punish_on_staggered_victim_is_1_25x_and_logged() {
    let mut m = duel(1.0);
    m.apply_status(M, Status::Stagger, 30, 1.0, H);
    run(&mut m, 5, atk(0), idle());
    assert_eq!(m.hp(M), 90.0, "8 * 1.25");
    assert!(m.log().to_json().contains("\"punish\":true"));
    let mut m = duel(1.0);
    run(&mut m, 5, atk(0), idle());
    assert!(!m.log().to_json().contains("\"punish\":true"));
}

#[test]
fn stagger_interrupts_windup_not_active() {
    let mut m = duel_sp(Species::Warrior, Rank::Grass, 1.0);
    m.step_with(idle(), atk(0));
    m.step_with(idle(), idle());
    assert_eq!(st(&m, M), S::Windup);
    m.apply_status(M, Status::Stagger, 10, 1.0, H);
    m.step_with(idle(), idle());
    assert_eq!(st(&m, M), S::Staggered);
    run(&mut m, 40, idle(), idle());
    assert_eq!(m.hp(H), 100.0, "cancelled club never lands");
    let mut m = duel(1.0);
    run(&mut m, 5, atk(0), idle());
    assert_eq!(st(&m, H), S::Active);
    m.apply_status(H, Status::Stagger, 10, 1.0, M);
    m.step_with(idle(), idle());
    assert_eq!(st(&m, H), S::Active, "committed swing not interrupted");
}

#[test]
fn guard_raise_gives_no_protection_for_6_ticks() {
    let mut m = duel(1.0);
    m.step_with(atk(0), blk());
    assert_eq!(st(&m, M), S::GuardRaise);
    run(&mut m, 4, idle(), blk());
    assert_eq!(m.hp(M), 92.0, "hit at step 5 lands in full during raise");
    run(&mut m, 2, idle(), blk());
    assert_eq!(st(&m, M), S::Blocking);
}

#[test]
fn blocked_hit_after_parry_window_loses_1_6() {
    let mut m = duel(1.0);
    run(&mut m, 8, idle(), blk());
    run(&mut m, 5, atk(0), blk());
    assert!((m.hp(M) - 98.4).abs() < 1e-3, "8 * 0.2, got {}", m.hp(M));
    assert_eq!(st(&m, M), S::Blocking);
}

#[test]
fn parry_blocks_all_and_staggers_attacker_14() {
    let mut m = duel(1.0);
    run(&mut m, 3, idle(), blk());
    run(&mut m, 5, atk(0), blk());
    assert_eq!(m.hp(M), 100.0);
    assert_eq!(st(&m, H), S::Staggered);
    let s = m.fx(H)[Status::Stagger.idx()];
    assert!((13..=14).contains(&s), "stagger {s}");
    assert!(m.log().to_json().contains("parry"));
}

#[test]
fn releasing_guard_enters_guard_recover_8_ignoring_inputs() {
    let mut m = duel(5.0);
    run(&mut m, 20, idle(), blk());
    assert_eq!(st(&m, M), S::Blocking);
    let mut n = 0;
    for _ in 0..20 {
        m.step_with(idle(), Action { attack: true, dodge: true, ..idle() });
        if st(&m, M) == S::GuardRecover { n += 1; } else if n > 0 { break; }
    }
    assert_eq!(n, 8);
}

#[test]
fn holding_block_drains_6_per_second_and_no_regen() {
    let mut m = duel(5.0);
    run(&mut m, 20, idle(), blk());
    let s20 = m.stamina(M);
    run(&mut m, 40, idle(), blk());
    assert!(s20 >= m.stamina(M), "no regen while blocking");
    assert!((87.0..=90.0).contains(&m.stamina(M)), "after 60 ticks {}", m.stamina(M));
}

#[test]
fn zero_stamina_drops_guard_into_guard_recover() {
    let mut m = duel(5.0);
    m.set_stamina(M, 1.0);
    let mut seen = false;
    for _ in 0..60 {
        m.step_with(idle(), blk());
        if st(&m, M) == S::GuardRecover { seen = true; break; }
    }
    assert!(seen);
    assert_ne!(st(&m, M), S::Stunned);
}

#[test]
fn block_arc_is_90_degrees_front_only() {
    let loss = |off_deg: f32| {
        let mut m = duel(1.0);
        m.set_facing(M, PI + off_deg.to_radians());
        run(&mut m, 10, idle(), blk());
        run(&mut m, 5, atk(0), blk());
        100.0 - m.hp(M)
    };
    assert!((loss(40.0) - 1.6).abs() < 1e-3);
    for off in [50.0, 100.0, 180.0] {
        assert!((loss(off) - 8.0).abs() < 1e-3, "off {off}");
    }
}

#[test]
fn block_arc_narrower_than_vision() {
    assert!((BLOCK_HALF_ARC - PI / 4.0).abs() < 1e-6);
    assert!(BLOCK_HALF_ARC < ArenaConfig::default().vision_fov / 2.0);
}

#[test]
fn blocked_hit_skips_effects_and_degrades_stun() {
    let go = |sp: Species, n: u32| {
        let mut m = duel_sp(sp, Rank::Grass, 1.0);
        run(&mut m, 10, blk(), idle());
        run(&mut m, n, blk(), atk(0));
        m
    };
    let m = go(Species::Rogue, 12);
    assert_eq!(m.fx(H)[Status::Bleed.idx()], 0, "blocked stab: no bleed");
    let m = go(Species::Mage, 24);
    assert_eq!(m.fx(H)[Status::Poison.idx()], 0);
    assert_eq!(m.fx(H)[Status::Confused.idx()], 0);
    let m = go(Species::Warrior, 16);
    assert_eq!(m.fx(H)[Status::Stun.idx()], 0, "blocked stun degrades");
    let s = m.fx(H)[Status::Stagger.idx()];
    assert!((7..=8).contains(&s), "stagger {s}");
}

#[test]
fn guard_break_stuns_20() {
    let mut m = duel(1.0);
    m.set_stamina(H, 10.0);
    run(&mut m, 10, blk(), idle());
    run(&mut m, 6, blk(), atk(0));
    assert_eq!(st(&m, H), S::Stunned);
    let s = m.fx(H)[Status::Stun.idx()];
    assert!((19..=20).contains(&s), "stun {s}");
    assert!(m.log().to_json().contains("guard_break"));
}

#[test]
fn dodge_iframes_ignore_effects() {
    let mut m = duel(1.0);
    m.step_with(idle(), dge());
    assert_eq!(st(&m, M), S::Dodging);
    m.apply_status(M, Status::Stun, 20, 1.0, H);
    m.apply_status(M, Status::Bleed, 90, 1.0, H);
    assert_eq!(m.fx(M), [0u32; 7]);
}

#[test]
fn dead_fighter_ignores_apply_status() {
    let mut m = duel(1.0);
    m.set_hp(M, 1.0);
    run(&mut m, 5, atk(0), idle());
    assert_eq!(st(&m, M), S::Dead);
    m.apply_status(M, Status::Stun, 20, 1.0, H);
    assert_eq!(m.fx(M), [0u32; 7]);
}
