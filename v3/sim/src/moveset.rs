// Baseline note (Gate 0, from plans/reports/2026-10-23T23-20-23_ver-v3-motion/index.md and git log):
// - Motion fix: smoothing is a first-order low-pass (config ACTION_SMOOTH 0.25) on the net's held target inside
//   duel.rs (not in robot.rs apply_targets); joint motors already rate-limit (MOTOR_KP, MOTOR_ACCEL). The
//   executor therefore feeds its min-jerk keyframe targets into that same single smoother (no second layer here).
// - Fitness gained a -W_JERK*jerk term (W_JERK 8); it stays, and the executor only adds a physical jerk energy cost.
// - Deathbattle: no timeout; HARD_CAP_TICKS 6000 is an emergency cap only; SUDDEN_DEATH_START 450 ticks, walls close
//   (WALL_SPEED, WALL_MIN_WIDTH) and the torso bleeds (SD_BLEED_BASE/GROWTH). No draw wording is relied upon here.
// - N_IN/N_OUT and DECISION_TICKS (3) are unchanged by that work; this workstream does not touch them.
//
//! Sword-and-shield move vocabulary: 13 moves plus Idle, phase timings, required joints and keyframe poses.
//! All timings are design numbers (seconds at speed param 0.5), tunable, not sourced frame data.
//! Poses are normalised 12-joint targets; phases interpolate between shared keyframes with a minimum-jerk quintic.
use crate::config::DELAY_MAX;
use crate::robot::{Joint, STAND_TARGETS};

pub const N_MOVES: usize = 14;
