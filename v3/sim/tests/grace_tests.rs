// WS-D grace metrics, curriculum, fitness-unchanged and training tests (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-D).
//
// INVENTED API:
//   mech_sim::grace (new file v3/sim/src/grace.rs). Metrics are MEASURED only, never rewarded.
//     pub fn whiff_rate(attack_starts: u32, whiffs: u32) -> f32;          // whiffs / starts, 0.0 when there are no starts
//     pub fn hits_per_swing(attack_starts: u32, body_hits: u32) -> f32;   // hits / starts, 0.0 when there are no starts
//     pub fn punish_rate(punish_hits: u32, open_windows: u32) -> f32;     // hits on an Open/Staggered opponent / windows entered, 0..=1, 0.0 if none
//     pub fn move_usage_entropy(counts: &[u32; 14]) -> f32;               // Shannon entropy in nats, 0 for none or one move
//     pub fn tip_jerk(tip: &[[f32; 2]], dt: f32) -> f32;                  // mean squared third derivative (finite differences)
//     pub struct GraceStats { pub jerk: f32, pub spacing_std: f32, pub hits_per_swing: f32, pub whiff_rate: f32, pub punish_rate: f32,
//         pub recovery_efficiency: f32, pub move_entropy: f32, pub usage: [f32; 14] /* shares, sum 1 or all 0 */,
//         pub parry_success_rate: f32, pub feint_bait_rate: f32, pub damage_per_energy: f32 }
//     pub fn match_grace(m: &Match, who: usize) -> GraceStats;            // after run_to_end, from the match's own records
//   mech_sim::curriculum:
//     #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum OppKind { Dummy, Skill(u8), SelfPlay, Hof }
//     pub fn opponent_mix(gen: u32) -> Vec<(OppKind, f32)>;               // weights sum to 1.0 (each > 0)
//   Training (mech_sim::evolve) with the existing TrainConfig::new(seed, pop, gens, out) and `threads`.
use mech_sim::config::PHYS_DT;
use mech_sim::curriculum::{opponent_mix, OppKind};
use mech_sim::duel::{Brain, Match};
use mech_sim::executor::Executor;
use mech_sim::fitness::*;
use mech_sim::grace::*;
use mech_sim::moveset::{min_jerk, MoveId, MoveParams, Phase, N_MOVES};
use mech_sim::robot::{Role, DT};
use std::path::PathBuf;

mod common;
use common::standing;

fn approx(a: f32, b: f32) { assert!((a - b).abs() < 1e-4, "{} vs {}", a, b); }
fn weight(gen: u32, k: OppKind) -> f32 { opponent_mix(gen).iter().filter(|(o, _)| *o == k).map(|(_, w)| *w).sum() }

// ---- pure metric helpers ----

#[test]
fn whiff_rate_is_1_for_swings_at_nothing_and_0_when_all_land() {
    approx(whiff_rate(10, 10), 1.0);
    approx(whiff_rate(10, 0), 0.0);
    approx(whiff_rate(8, 2), 0.25);
    approx(whiff_rate(0, 0), 0.0);
}

#[test]
fn hits_per_swing_counts_body_hits_over_attack_starts() {
    approx(hits_per_swing(10, 4), 0.4);
    approx(hits_per_swing(0, 0), 0.0);
}

#[test]
fn punish_rate_counts_only_hits_on_open_or_staggered_windows() {
    approx(punish_rate(0, 5), 0.0);
    approx(punish_rate(3, 6), 0.5);
    approx(punish_rate(0, 0), 0.0);
    assert!(punish_rate(9, 3) <= 1.0, "clamped to 0..=1");
}

#[test]
fn move_usage_entropy_bounds() {
    let mut c = [0u32; N_MOVES];
    approx(move_usage_entropy(&c), 0.0);
    c[5] = 20;
    approx(move_usage_entropy(&c), 0.0);
    c = [3u32; N_MOVES];
    approx(move_usage_entropy(&c), (N_MOVES as f32).ln());
    c[0] = 30;
    let e = move_usage_entropy(&c);
    assert!(e > 0.0 && e < (N_MOVES as f32).ln());
}

fn curve(f: impl Fn(f32) -> f32) -> Vec<[f32; 2]> { (0..=40).map(|i| [f(i as f32 / 40.0), 0.0]).collect() }

#[test]
fn min_jerk_path_is_smoother_than_a_ramp_with_velocity_kinks() {
    let smooth = tip_jerk(&curve(|t| 2.0 * min_jerk(t)), PHYS_DT);
    let ramp = tip_jerk(&curve(|t| 2.0 * t), PHYS_DT);
    assert!(smooth.is_finite() && ramp.is_finite());
    // A ramp has zero jerk in its interior but the ramp-and-hold of a step-target move does not.
    let step = tip_jerk(&curve(|t| if t < 0.5 { 4.0 * t } else { 2.0 }), PHYS_DT);
    assert!(smooth < step, "min-jerk {} vs step {}", smooth, step);
}

#[test]
fn tip_jerk_is_zero_for_constant_velocity_and_scale_sensitive() {
    approx(tip_jerk(&curve(|t| 3.0 * t), PHYS_DT), 0.0);
    let small = tip_jerk(&curve(|t| min_jerk(t)), PHYS_DT);
    let big = tip_jerk(&curve(|t| 2.0 * min_jerk(t)), PHYS_DT);
    assert!(big > small * 3.5 && big < small * 4.5, "jerk scales with amplitude squared: {} {}", small, big);
}

fn tip_trace(drive_executor: bool) -> Vec<[f32; 2]> {
    let mut r = standing(Role::Hero, 3.0, 1.0);
    let mut ex = Executor::new();
    let p = MoveParams { reach: 0.5, speed: 0.5, arc: 0.5, delay: 0.0 };
    let mut out = Vec::new();
    // Joint targets of the Active pose applied as one raw step (what the old raw-joint net did).
    let step = mech_sim::moveset::pose_at(MoveId::Overhead, &p, Phase::Active, 0.5);
    for i in 0..60 {
        let a = if drive_executor { ex.step(&r, if i == 0 { Some((MoveId::Overhead, p)) } else { None }, DT).targets } else { step };
        r.apply_targets(&a, DT);
        out.push(r.sword_capsule().b);
    }
    out
}

#[test]
fn executor_driven_swing_has_lower_tip_jerk_than_a_step_target_swing() {
    let smooth = tip_jerk(&tip_trace(true), DT);
    let raw = tip_jerk(&tip_trace(false), DT);
    assert!(smooth.is_finite() && raw.is_finite());
    assert!(smooth < raw, "executor {} vs step {}", smooth, raw);
}

// ---- match level metrics ----

fn sample() -> (Match, GraceStats, GraceStats) {
    let mut m = Match::new(4, Brain::Skill(3), Brain::Skill(2));
    m.run_to_end();
    let a = match_grace(&m, 0);
    let b = match_grace(&m, 1);
    (m, a, b)
}

fn check(g: &GraceStats) {
    for v in [g.jerk, g.spacing_std, g.hits_per_swing, g.whiff_rate, g.punish_rate, g.recovery_efficiency, g.move_entropy,
        g.parry_success_rate, g.feint_bait_rate, g.damage_per_energy] {
        assert!(v.is_finite() && v >= 0.0, "{:?}", v);
    }
    for v in [g.whiff_rate, g.punish_rate, g.parry_success_rate, g.feint_bait_rate] { assert!(v <= 1.0 + 1e-5); }
    assert!(g.move_entropy <= (N_MOVES as f32).ln() + 1e-4);
    let s: f32 = g.usage.iter().sum();
    assert!(s == 0.0 || (s - 1.0).abs() < 1e-3, "usage sums to {}", s);
    assert!(g.usage.iter().all(|u| (0.0..=1.0 + 1e-5).contains(u)));
}

#[test]
fn match_grace_is_finite_and_in_range_for_both_fighters() {
    let (_, a, b) = sample();
    check(&a);
    check(&b);
    assert!(a.usage.iter().sum::<f32>() > 0.5, "a skillbot fight must record move usage");
}

#[test]
fn match_grace_is_deterministic() {
    let (_, a1, _) = sample();
    let (_, a2, _) = sample();
    assert_eq!(a1.usage, a2.usage);
    assert_eq!((a1.jerk, a1.whiff_rate, a1.punish_rate, a1.hits_per_swing), (a2.jerk, a2.whiff_rate, a2.punish_rate, a2.hits_per_swing));
}

#[test]
fn a_spammer_against_a_dummy_that_never_defends_lands_hits_a_spammer_in_the_void_whiffs() {
    let mut m = Match::new(2, Brain::Spam(MoveId::Overhead), Brain::Dummy);
    m.run_to_end();
    let g = match_grace(&m, 0);
    assert!(g.hits_per_swing > 0.0 && g.whiff_rate < 1.0, "spammer vs dummy hits: {:?}", g.whiff_rate);
    assert_eq!(g.usage[MoveId::Overhead as usize] > 0.99, true, "only Overhead used");
}

// ---- fitness unchanged ----

#[test]
fn fitness_weights_are_unchanged() {
    approx(W_WIN, 4.0);
    approx(W_DEALT, 3.0);
    approx(W_TAKEN, 2.5);
    approx(W_DEFENCE, 2.0);
    approx(W_IDLE_GUARD, 1.0);
    approx(W_PASSIVITY, 1.0);
    approx(W_CHAIN, 0.5);
    approx(W_JERK, 8.0); // baseline from the motion fix, must not change
    approx(KO_BONUS, 1.0);
    assert_eq!(MAX_CREDITED_THREATS, 6);
}

#[test]
fn score_stats_has_no_new_term() {
    // Exhaustive struct literal: adding a field to MatchStats (a new reward input beyond the baseline jerk term) breaks compilation here.
    let s = MatchStats { win: true, draw: false, ko: true, dmg_dealt_norm: 0.5, dmg_taken_norm: 0.2, defence_credit: 0.5, idle_guard: 0.1, passivity: 0.2, chain_quality: 0.4, jerk: 0.05 };
    let expect = 4.0 + 1.0 + 3.0 * 0.5 - 2.5 * 0.2 + 2.0 * 0.5 - 0.1 - 0.2 + 0.5 * 0.4 - 8.0 * 0.05;
    approx(score_stats(&s), expect);
    // Zeroing every existing term leaves exactly the win bonus: nothing else contributes.
    let z = MatchStats { win: true, draw: false, ko: false, dmg_dealt_norm: 0.0, dmg_taken_norm: 0.0, defence_credit: 0.0, idle_guard: 0.0, passivity: 0.0, chain_quality: 0.0, jerk: 0.0 };
    approx(score_stats(&z), 4.0);
}

// ---- curriculum ----

fn sum(gen: u32) -> f32 { opponent_mix(gen).iter().map(|(_, w)| *w).sum() }

#[test]
fn mix_weights_sum_to_one_everywhere() {
    for g in [0, 15, 29, 30, 54, 55, 79, 80, 150, 200] {
        approx(sum(g), 1.0);
        assert!(opponent_mix(g).iter().all(|(_, w)| *w > 0.0), "gen {}", g);
    }
}
