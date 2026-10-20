//! Tunable constants. Later workstreams add the robot, combat and fitness constants here.

/// Physics step, seconds (60 Hz).
pub const PHYS_DT: f32 = 1.0 / 60.0;
/// The net decides every this many physics ticks (20 Hz).
pub const DECISION_TICKS: u32 = 3;
/// Emergency safety net only. A fight that reaches it is a capped mutual loss (scored far below any
/// death-loss, see `CAPPED_SCORE`) and is flagged `capped` in the log. Nothing else forces an ending.
pub const HARD_CAP_TICKS: u32 = 6000;
/// Training-only stall penalty (fitness, never an in-fight rule): a stretch with no hit by either side
/// costs nothing for this many ticks (3 s), then `((len - grace) / STALL_REF_TICKS)^1.5` units.
pub const STALL_GRACE_TICKS: u32 = 180;
pub const STALL_REF_TICKS: f32 = 1200.0;
/// Fitness cost per stall unit, bounded by `W_STALL_MAX` (kept small enough that a capped fight stays worst).
pub const W_STALL: f32 = 1.0;
pub const W_STALL_MAX: f32 = 2.0;
/// Fitness score of a capped fight, for both fighters: below a draw and any death-loss.
pub const CAPPED_SCORE: f32 = -12.0;
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

// ---- Sword vocabulary executor (WS-A). Design numbers, tunable, not sourced. ----
/// Energy cost of a Feint (cancels a windup).
pub const FEINT_COST: f32 = 8.0;
/// Energy cost of the full Drag delay (scaled by the delay param 0..1).
pub const DRAG_COST: f32 = 3.0;
/// Energy cost of chaining a move out of the previous recovery.
pub const CHAIN_COST: f32 = 5.0;
/// Energy cost of cancelling a windup with a threatened Backstep.
pub const BACKSTEP_CANCEL_COST: f32 = 5.0;
/// Energy per unit of |second derivative of the joint targets| (smooth moves are cheaper: physics, not reward).
pub const JERK_COST: f32 = 0.0004;
/// A Feint cancels only before this fraction of the windup.
pub const FEINT_CANCEL_FRAC: f32 = 0.6;
/// Chain window: the last this-fraction of Recovery (progress >= 1 - CHAIN_FRAC).
pub const CHAIN_FRAC: f32 = 0.25;
/// Seconds to relax back to the ready pose after a cancelled windup.
pub const CANCEL_RECOVER: f32 = 0.15;
/// Riposte windup when its window is open (seconds at speed 0.5).
pub const RIPOSTE_WINDUP_OPEN: f32 = 0.15;
/// Extra windup hold at delay param 1.0 (Drag), seconds.
pub const DELAY_MAX: f32 = 0.2;
