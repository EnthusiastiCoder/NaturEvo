//! Tunable constants. Later workstreams add the robot, combat and fitness constants here.

/// Physics step, seconds (60 Hz).
pub const PHYS_DT: f32 = 1.0 / 60.0;
/// The net decides every this many physics ticks (20 Hz).
pub const DECISION_TICKS: u32 = 3;
/// Emergency safety net only: fights are death battles and should never reach this many ticks.
/// A capped fight is a Draw (never an hp-lead win) and is reported.
pub const HARD_CAP_TICKS: u32 = 6000;
/// Grace period before sudden death (ticks, 7.5 s). After it the walls close in and the torso bleeds
/// at an escalating rate, which forces engagement so a fight always ends in a death.
pub const SUDDEN_DEATH_START: u32 = 450;
/// Each wall closes in at this speed (m/s) after onset.
pub const WALL_SPEED: f32 = 0.6;
/// The walls stop closing at this arena width (m).
pub const WALL_MIN_WIDTH: f32 = 1.6;
/// Sudden-death torso bleed, hp/s, at onset; grows by `SD_BLEED_GROWTH` (fraction of base) per second.
pub const SD_BLEED_BASE: f32 = 4.0;
pub const SD_BLEED_GROWTH: f32 = 0.25;
/// Motor velocity gain (1/s): desired joint speed = MOTOR_KP * angle error, capped at the joint max speed.
pub const MOTOR_KP: f32 = 14.0;
/// Motor angular acceleration limit, rad/s^2 (joint speed ramps, never steps).
pub const MOTOR_ACCEL: f32 = 100.0;
/// Per-tick low-pass factor on the net's held target (first order, 0..1).
pub const ACTION_SMOOTH: f32 = 0.25;
/// Arena width in metres.
pub const ARENA_WIDTH: f32 = 14.0;

// ---- Robot body (WS1) ----
/// Cap applied to a raw impact before zone damage and posture.
pub const IMPACT_CAP: f32 = 200.0;
/// Reference impact for defence credit normalisation.
pub const IMPACT_REF: f32 = 60.0;
/// Fitness score of a draw (strictly worse than any loss).
pub const DRAW_SCORE: f32 = -6.0;
/// Energy pool maximum.
pub const ENERGY_MAX: f32 = 100.0;
/// Energy regenerated per second when idle.
pub const ENERGY_REGEN: f32 = 8.0;
/// Energy drained per second while the shield guard is held.
pub const GUARD_DRAIN: f32 = 5.0;
/// Energy cost per radian of joint motion.
pub const MOTOR_COST: f32 = 0.5;
/// Torque scale cap when the energy pool is empty.
pub const EMPTY_ENERGY_TORQUE: f32 = 0.3;
/// Posture decay per second (after `POSTURE_DECAY_DELAY` without a hit).
pub const POSTURE_DECAY: f32 = 15.0;
pub const POSTURE_DECAY_DELAY: f32 = 0.5;
pub const STAGGER_TIME: f32 = 0.8;
pub const STAGGER_RESET: f32 = 40.0;
pub const FALL_TIME: f32 = 1.5;
/// Seconds of negative balance margin before the robot falls.
pub const FALL_MARGIN_TIME: f32 = 0.4;
/// Damage multiplier while staggered (exposed) or fallen.
pub const EXPOSED_MULT: f32 = 1.5;
/// Maximum walking speed, m/s.
pub const DRIVE_SPEED: f32 = 2.0;

// ---- Body geometry: single source of truth, written into the fight log "geometry" block ----
/// Head hit radius, metres (viewer draws exactly this).
pub const HEAD_R: f32 = 0.23;
/// Neck hit radius, metres. The neck is NOT a separate zone: a hit on it damages the Head part
/// (same zone multiplier), so a blade cannot slip between head and torso.
pub const NECK_R: f32 = 0.09;
/// Shield plate length (metres), perpendicular to the forearm axis.
pub const SHIELD_LEN: f32 = 0.7;
/// Shield plate centre as a fraction of the forearm from the elbow.
pub const SHIELD_ALONG: f32 = 0.62;
/// Shield plate half-thickness (hit radius).
pub const SHIELD_R: f32 = 0.05;
pub const SWORD_LEN: f32 = 2.2;
pub const SWORD_R: f32 = 0.05;
