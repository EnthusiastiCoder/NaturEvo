//! Tunable constants. Later workstreams add the robot, combat and fitness constants here.

/// Physics step, seconds (60 Hz).
pub const PHYS_DT: f32 = 1.0 / 60.0;
/// The net decides every this many physics ticks (20 Hz).
pub const DECISION_TICKS: u32 = 3;
/// Maximum match length in physics ticks (15 s).
pub const MAX_TICKS: u32 = 900;
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
