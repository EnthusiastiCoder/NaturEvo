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

use Joint::*;

static IDLE_SPEC: MoveSpec = MoveSpec { windup: 0.0, active: 0.0, recovery: 0.0, required: &[], energy: 0.0, can_whiff: false };
static OVERHEAD: MoveSpec = MoveSpec { windup: 0.35, active: 0.10, recovery: 0.40, required: &[HipR, Waist, ShoulderR, ElbowR], energy: 7.0, can_whiff: true };
static DIAG_DOWN: MoveSpec = MoveSpec { windup: 0.30, active: 0.10, recovery: 0.35, required: &[HipR, Waist, ShoulderR, ElbowR], energy: 6.0, can_whiff: true };
static DIAG_UP: MoveSpec = MoveSpec { windup: 0.25, active: 0.10, recovery: 0.30, required: &[HipR, Waist, ShoulderR, ElbowR], energy: 5.0, can_whiff: true };
static SWEEP: MoveSpec = MoveSpec { windup: 0.30, active: 0.15, recovery: 0.45, required: &[HipR, Waist, ShoulderR, ElbowR], energy: 8.0, can_whiff: true };
static THRUST: MoveSpec = MoveSpec { windup: 0.20, active: 0.08, recovery: 0.35, required: &[HipR, KneeR, Waist, ShoulderR, ElbowR], energy: 6.0, can_whiff: true };
static LOW_CUT: MoveSpec = MoveSpec { windup: 0.30, active: 0.12, recovery: 0.40, required: &[HipR, KneeR, Waist, ShoulderR, ElbowR], energy: 6.0, can_whiff: true };
static FEINT: MoveSpec = MoveSpec { windup: 0.10, active: 0.05, recovery: 0.15, required: &[Waist, ShoulderR, ElbowR], energy: 8.0, can_whiff: false };
static PARRY: MoveSpec = MoveSpec { windup: 0.05, active: 0.15, recovery: 0.30, required: &[ShoulderR, ElbowR, Waist], energy: 3.0, can_whiff: true };
static GUARD: MoveSpec = MoveSpec { windup: 0.10, active: 0.40, recovery: 0.10, required: &[ShoulderL, ElbowL], energy: 2.0, can_whiff: false };
static RIPOSTE: MoveSpec = MoveSpec { windup: 0.30, active: 0.10, recovery: 0.30, required: &[HipR, Waist, ShoulderR, ElbowR], energy: 6.0, can_whiff: true };
static SHIELD_BASH: MoveSpec = MoveSpec { windup: 0.25, active: 0.10, recovery: 0.35, required: &[Waist, ShoulderL, ElbowL], energy: 6.0, can_whiff: true };
static KICK: MoveSpec = MoveSpec { windup: 0.15, active: 0.08, recovery: 0.30, required: &[HipR, KneeR, AnkleR], energy: 5.0, can_whiff: true };
static BACKSTEP: MoveSpec = MoveSpec { windup: 0.20, active: 0.10, recovery: 0.25, required: &[HipL, HipR, KneeL, KneeR, AnkleL, AnkleR], energy: 4.0, can_whiff: false };

pub fn spec(id: MoveId) -> &'static MoveSpec {
    match id {
        MoveId::Idle => &IDLE_SPEC,
        MoveId::Overhead => &OVERHEAD,
        MoveId::DiagDown => &DIAG_DOWN,
        MoveId::DiagUp => &DIAG_UP,
        MoveId::Sweep => &SWEEP,
        MoveId::Thrust => &THRUST,
        MoveId::LowCut => &LOW_CUT,
        MoveId::Feint => &FEINT,
        MoveId::Parry => &PARRY,
        MoveId::Guard => &GUARD,
        MoveId::Riposte => &RIPOSTE,
        MoveId::ShieldBash => &SHIELD_BASH,
        MoveId::Kick => &KICK,
        MoveId::Backstep => &BACKSTEP,
    }
}

/// Phase duration multiplier from the speed param: 1.4 at 0, 1.0 at 0.5, 0.7 at 1.
pub fn speed_mult(speed: f32) -> f32 {
    let s = speed.clamp(0.0, 1.0);
    if s <= 0.5 {
        1.4 - 0.8 * s
    } else {
        1.0 - 0.6 * (s - 0.5)
    }
}

/// [windup (including the Drag delay), active, recovery] in seconds.
pub fn phase_durations(id: MoveId, p: &MoveParams) -> [f32; 3] {
    let s = spec(id);
    let m = speed_mult(p.speed);
    [s.windup * m + DELAY_MAX * p.delay.clamp(0.0, 1.0), s.active * m, s.recovery * m]
}

/// Strike amplitude at speed 0 (1.0 at speed 1).
pub const STRIKE_AMP_SLOW: f32 = 0.4;

/// Minimum-jerk quintic, 0 -> 1 with zero velocity and acceleration at both ends.
pub fn min_jerk(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    10.0 * p.powi(3) - 15.0 * p.powi(4) + 6.0 * p.powi(5)
}

/// Ready pose: the standing pose (the sword arm is not pre-raised, so a windup travels a physically feasible arc).
pub const REST: [f32; 12] = STAND_TARGETS;

fn lerp(a: f32, b: f32, s: f32) -> f32 {
    a + (b - a) * s
}

fn mk(edits: &[(Joint, f32)]) -> [f32; 12] {
    let mut t = REST;
    for (j, v) in edits {
        t[*j as usize] = v.clamp(-1.0, 1.0);
    }
    t
}

/// (windup pose, strike pose) keyframes of a move for the given params.
fn keyframes(id: MoveId, p: &MoveParams) -> ([f32; 12], [f32; 12]) {
    let r = p.reach.clamp(0.0, 1.0);
    let a = p.arc.clamp(0.0, 1.0);
    match id {
        // Scheitelhau: vertical cut from above. Arc tilts the waist follow-through.
        MoveId::Overhead => (
            mk(&[(ShoulderR, 0.3), (ElbowR, 0.4), (Waist, -0.4), (HipR, -0.1)]),
            mk(&[(ShoulderR, 0.1), (ElbowR, lerp(-0.2, -1.0, r)), (Waist, lerp(0.2, 0.6, a)), (HipR, 0.0)]),
        ),
        // Zornhau: diagonal cut, arc tilts the angle.
        MoveId::DiagDown => (
            mk(&[(ShoulderR, 0.35), (ElbowR, 0.0), (Waist, -0.5), (HipR, -0.1)]),
            mk(&[(ShoulderR, lerp(0.1, -0.5, a)), (ElbowR, lerp(-0.1, -0.8, r)), (Waist, 0.4), (HipR, 0.0)]),
        ),
        // Rising cut from low.
        MoveId::DiagUp => (
            mk(&[(ShoulderR, -0.8), (ElbowR, 0.1), (Waist, -0.4), (HipR, -0.2)]),
            mk(&[(ShoulderR, lerp(-0.2, 0.3, a)), (ElbowR, lerp(-0.2, -0.8, r)), (Waist, 0.4), (HipR, 0.0)]),
        ),
        // Zwerchhau: wide horizontal arc, arc picks the height.
        MoveId::Sweep => (
            mk(&[(ShoulderR, lerp(-0.1, 0.4, a)), (ElbowR, 0.0), (Waist, -0.7), (HipR, -0.1)]),
            mk(&[(ShoulderR, lerp(-0.1, 0.4, a)), (ElbowR, lerp(-0.1, -0.8, r)), (Waist, 0.7), (HipR, 0.0)]),
        ),
        // Pflug: straight thrust with a lunge; reach straightens the arm.
        MoveId::Thrust => (
            mk(&[(ShoulderR, -0.2), (ElbowR, 0.0), (Waist, -0.3), (HipR, -0.3)]),
            mk(&[(ShoulderR, lerp(-0.2, 0.2, a)), (ElbowR, lerp(-0.3, -0.8, r)), (Waist, 0.4), (HipR, 0.0), (KneeR, -0.6)]),
        ),
        // Krumphau: low cut at the legs.
        MoveId::LowCut => (
            mk(&[(ShoulderR, 0.1), (ElbowR, 0.1), (Waist, -0.5), (HipR, -0.1)]),
            mk(&[(ShoulderR, lerp(-0.8, -0.4, a)), (ElbowR, lerp(-0.2, -0.8, r)), (Waist, 0.6), (HipR, 0.2), (KneeR, -0.7)]),
        ),
        // Request-only: never runs as a move of its own.
        MoveId::Idle | MoveId::Feint => (REST, REST),
        // Deflect: sword comes up and across in front.
        MoveId::Parry => (
            mk(&[(ShoulderR, -0.2), (ElbowR, 0.0), (Waist, -0.1)]),
            mk(&[(ShoulderR, lerp(0.1, 0.5, a)), (ElbowR, lerp(-0.1, -0.5, r)), (Waist, 0.2)]),
        ),
        // Shield raised in front (GUARD_TARGETS on the left arm).
        MoveId::Guard => (
            mk(&[(ShoulderL, 0.1), (ElbowL, 0.29)]),
            mk(&[(ShoulderL, lerp(0.05, 0.3, a)), (ElbowL, lerp(0.2, 0.5, r))]),
        ),
        // Quick counter cut after a parry; a straight-ish stab.
        MoveId::Riposte => (
            mk(&[(ShoulderR, 0.2), (ElbowR, 0.3), (Waist, -0.3), (HipR, -0.2)]),
            mk(&[(ShoulderR, lerp(0.0, 0.5, a)), (ElbowR, lerp(-0.3, -1.0, r)), (Waist, 0.4), (HipR, 0.1)]),
        ),
        // Shield plate driven forward.
        MoveId::ShieldBash => (
            mk(&[(ShoulderL, -0.8), (ElbowL, 0.6), (Waist, -0.3)]),
            mk(&[(ShoulderL, lerp(0.0, 0.4, a)), (ElbowL, lerp(0.2, -0.3, r)), (Waist, 0.3)]),
        ),
        // Foot strike with the right leg.
        MoveId::Kick => (
            mk(&[(HipR, -0.4), (KneeR, 0.0), (AnkleR, -0.2)]),
            mk(&[(HipR, lerp(0.1, 0.5, r)), (KneeR, -0.8), (AnkleR, 0.2)]),
        ),
        // Crouch then push off backwards (the drive is added by the executor).
        MoveId::Backstep => (
            mk(&[(HipL, -0.4), (HipR, -0.4), (KneeL, 0.0), (KneeR, 0.0)]),
            mk(&[(HipL, lerp(-0.3, 0.1, r)), (HipR, lerp(-0.3, 0.1, r)), (KneeL, -0.5), (KneeR, -0.5)]),
        ),
    }
}

fn blend(a: &[f32; 12], b: &[f32; 12], s: f32) -> [f32; 12] {
    let mut o = [0.0; 12];
    for i in 0..12 {
        o[i] = lerp(a[i], b[i], s).clamp(-1.0, 1.0);
    }
    o
}
