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

/// Fixed order: it is the net output index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveId {
    Idle = 0,
    Overhead = 1,
    DiagDown = 2,
    DiagUp = 3,
    Sweep = 4,
    Thrust = 5,
    LowCut = 6,
    Feint = 7,
    Parry = 8,
    Guard = 9,
    Riposte = 10,
    ShieldBash = 11,
    Kick = 12,
    Backstep = 13,
}

impl MoveId {
    pub const ALL: [MoveId; N_MOVES] = [
        MoveId::Idle,
        MoveId::Overhead,
        MoveId::DiagDown,
        MoveId::DiagUp,
        MoveId::Sweep,
        MoveId::Thrust,
        MoveId::LowCut,
        MoveId::Feint,
        MoveId::Parry,
        MoveId::Guard,
        MoveId::Riposte,
        MoveId::ShieldBash,
        MoveId::Kick,
        MoveId::Backstep,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> MoveId {
        MoveId::ALL[i.min(N_MOVES - 1)]
    }

    pub fn name(self) -> &'static str {
        match self {
            MoveId::Idle => "Idle",
            MoveId::Overhead => "Overhead",
            MoveId::DiagDown => "DiagDown",
            MoveId::DiagUp => "DiagUp",
            MoveId::Sweep => "Sweep",
            MoveId::Thrust => "Thrust",
            MoveId::LowCut => "LowCut",
            MoveId::Feint => "Feint",
            MoveId::Parry => "Parry",
            MoveId::Guard => "Guard",
            MoveId::Riposte => "Riposte",
            MoveId::ShieldBash => "ShieldBash",
            MoveId::Kick => "Kick",
            MoveId::Backstep => "Backstep",
        }
    }
}

/// Continuous move parameters, each 0..=1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveParams {
    pub reach: f32,
    pub speed: f32,
    pub arc: f32,
    pub delay: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Windup,
    Active,
    Recovery,
}

impl Phase {
    pub fn index(self) -> usize {
        self as usize
    }
}

pub struct MoveSpec {
    /// Seconds at speed 0.5, delay 0.
    pub windup: f32,
    pub active: f32,
    pub recovery: f32,
    /// Proximal to distal. A disabled/destroyed part of any of these makes the move impossible.
    pub required: &'static [Joint],
    /// Base energy at speed 0.5 (on top of the motor cost).
    pub energy: f32,
    /// Active ending with no contact counts as a whiff (Guard, Backstep and Feint cannot whiff).
    pub can_whiff: bool,
}
