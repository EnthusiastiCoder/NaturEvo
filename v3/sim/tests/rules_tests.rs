// WS-B rules tests (plan 2026-10-23_v3-sword-vocabulary.md, "Rules that make skill lethal", Test cases WS-B).
//
// INVENTED API (module `mech_sim::rules`, new file v3/sim/src/rules.rs). Pure data and functions, no RNG, no net or
// fitness access:
//   pub const OPEN_MULT: f32 = 1.30;            // damage x while Open after a whiff
//   pub const OPEN_POSTURE_MULT: f32 = 1.5;     // posture gain x while Open after a whiff
//   pub const COUNTER_HIT_MULT: f32 = 1.15;     // victim was in its own Windup/Active
//   pub const MULT_CAP: f32 = 1.75;
//   pub const RIPOSTE_WINDOW: f32 = 0.6;  pub const RIPOSTE_MULT: f32 = 1.5;
//   pub const RIPOSTE_WINDUP_FAST: f32 = 0.15;  pub const RIPOSTE_WINDUP_SLOW: f32 = 0.30;
//   pub const PARRY_MISS_RECOVERY: f32 = 0.30;  pub const PARRY_MISS_OPEN_MULT: f32 = 1.2;
//   pub const SHIELD_BASH_POSTURE: f32 = 35.0;  pub const GUARD_BREAK_TIME: f32 = 0.4;  pub const GUARD_BREAK_OPEN_MULT: f32 = 1.25;
//   pub const HEAVY_BLOCK_POSTURE_MULT: f32 = 1.4;
//   pub const BACKSTEP_COUNTER_POSTURE: f32 = 30.0;  pub const BACKSTEP_COUNTER_OPEN: f32 = 0.3;
//   pub const FEINT_COST: f32 = 8.0;  pub const DRAG_COST: f32 = 3.0;  pub const CHAIN_COST: f32 = 5.0;
//
//   #[derive(Clone, Debug, PartialEq, Default)] pub struct RuleState;   // per fighter, timers in seconds
//   impl RuleState {
//     pub fn new() -> RuleState;
//     pub fn tick(&mut self, dt: f32);                       // counts every timer down, never below 0
//     pub fn begin_recovery(&mut self, whiffed: bool, recovery: f32);  // whiffed => Open for `recovery` s at OPEN_MULT, else no Open
//     pub fn on_parry_success(&mut self);                    // opens the riposte window (RIPOSTE_WINDOW)
//     pub fn on_parry_miss(&mut self);                       // Open for PARRY_MISS_RECOVERY s at PARRY_MISS_OPEN_MULT
//     pub fn is_open(&self) -> bool;   pub fn open_remaining(&self) -> f32;   pub fn open_mult(&self) -> f32; // 1.0 when not open
//     pub fn riposte_open(&self) -> bool;   pub fn riposte_remaining(&self) -> f32;
//     pub fn guard_broken_remaining(&self) -> f32;
//   }
//   pub fn combine(mults: &[f32]) -> f32;      // max(1.0, max of mults), capped at MULT_CAP (never multiplies)
//   /// Damage multiplier on a hit against the victim: `st` is the VICTIM's RuleState.
//   pub fn punish_mult(st: &RuleState, victim_in_windup_or_active: bool, victim_staggered: bool, riposte_hit: bool) -> f32;
//   pub fn riposte_windup(st: &RuleState) -> f32;   // attacker's RuleState: 0.15 inside its window else 0.30
//   pub fn guard_break(defender: &mut Robot, def_st: &mut RuleState);       // ShieldBash landed on a guarding/blocking defender
//   pub fn heavy_block_posture(m: MoveId, base_posture: f32) -> f32;        // x1.4 for Overhead and Sweep only
//   pub fn backstep_counter(attacker: &mut Robot, att_st: &mut RuleState);  // Backstep evaded a Thrust that would have connected
use mech_sim::moveset::MoveId;
use mech_sim::robot::{Robot, Role};
use mech_sim::rules::*;

fn approx(a: f32, b: f32) { assert!((a - b).abs() < 1e-4, "{} vs {}", a, b); }

#[test]
fn constants_match_the_plan() {
    approx(OPEN_MULT, 1.30);
    approx(OPEN_POSTURE_MULT, 1.5);
    approx(COUNTER_HIT_MULT, 1.15);
    approx(MULT_CAP, 1.75);
    approx(RIPOSTE_WINDOW, 0.6);
    approx(RIPOSTE_MULT, 1.5);
    approx(PARRY_MISS_RECOVERY, 0.30);
    approx(PARRY_MISS_OPEN_MULT, 1.2);
    approx(SHIELD_BASH_POSTURE, 35.0);
    approx(GUARD_BREAK_TIME, 0.4);
    approx(GUARD_BREAK_OPEN_MULT, 1.25);
    approx(HEAVY_BLOCK_POSTURE_MULT, 1.4);
    approx(BACKSTEP_COUNTER_POSTURE, 30.0);
    approx(FEINT_COST, 8.0);
    approx(DRAG_COST, 3.0);
    approx(CHAIN_COST, 5.0);
}

#[test]
fn whiff_recovery_is_open_with_x1_30_damage() {
    let mut st = RuleState::new();
    st.begin_recovery(true, 0.40);
    assert!(st.is_open());
    approx(st.open_mult(), 1.30);
    approx(punish_mult(&st, false, false, false), 1.30);
    st.tick(0.41);
    assert!(!st.is_open());
    approx(punish_mult(&st, false, false, false), 1.0);
}

#[test]
fn a_landed_move_leaves_no_open_window() {
    let mut st = RuleState::new();
    st.begin_recovery(false, 0.40);
    assert!(!st.is_open());
    approx(st.open_mult(), 1.0);
    approx(punish_mult(&st, false, false, false), 1.0);
}

#[test]
fn counter_hit_x1_15_only_during_own_windup_or_active() {
    let st = RuleState::new();
    approx(punish_mult(&st, true, false, false), 1.15);
    approx(punish_mult(&st, false, false, false), 1.0);
}

#[test]
fn multipliers_combine_by_max_not_product() {
    let mut st = RuleState::new();
    st.begin_recovery(true, 0.4);
    approx(punish_mult(&st, true, false, false), 1.30); // open 1.30 vs counter 1.15
    approx(punish_mult(&st, true, true, false), 1.5);   // staggered exposed 1.5 wins
    approx(punish_mult(&st, false, true, false), 1.5);
}

#[test]
fn combine_takes_max_and_caps_at_1_75() {
    approx(combine(&[]), 1.0);
    approx(combine(&[0.5]), 1.0);
    approx(combine(&[1.3, 1.15, 1.5]), 1.5);
    approx(combine(&[1.5, 2.2]), 1.75);
    approx(combine(&[3.0, 3.0, 3.0]), 1.75);
}

#[test]
fn punish_mult_never_exceeds_the_cap() {
    let mut st = RuleState::new();
    st.begin_recovery(true, 0.4);
    st.on_parry_success();
    for a in [false, true] { for b in [false, true] { for c in [false, true] {
        let m = punish_mult(&st, a, b, c);
        assert!((1.0..=1.75 + 1e-5).contains(&m), "{}", m);
    } } }
}

#[test]
fn parry_success_opens_a_0_6_s_riposte_window_that_expires() {
    let mut st = RuleState::new();
    assert!(!st.riposte_open());
    st.on_parry_success();
    assert!(st.riposte_open());
    approx(st.riposte_remaining(), 0.6);
    st.tick(0.3);
    assert!(st.riposte_open());
    approx(st.riposte_remaining(), 0.3);
    st.tick(0.31);
    assert!(!st.riposte_open());
    approx(st.riposte_remaining(), 0.0);
}

#[test]
fn riposte_windup_is_fast_inside_the_window_and_ordinary_outside() {
    let mut st = RuleState::new();
    approx(riposte_windup(&st), 0.30);
    st.on_parry_success();
    approx(riposte_windup(&st), 0.15);
    st.tick(0.7);
    approx(riposte_windup(&st), 0.30);
}

#[test]
fn riposte_hit_deals_x1_5_on_staggered_or_open_targets_only() {
    let neutral = RuleState::new();
    approx(punish_mult(&neutral, false, true, true), 1.5);
    let mut open = RuleState::new();
    open.begin_recovery(true, 0.4);
    approx(punish_mult(&open, false, false, true), 1.5);
    approx(punish_mult(&neutral, false, false, true), 1.0);
}
