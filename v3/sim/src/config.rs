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
