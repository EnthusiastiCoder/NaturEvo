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
