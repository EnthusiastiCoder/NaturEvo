//! Tunable constants. Later workstreams add the robot, combat and fitness constants here.

/// Physics step, seconds (60 Hz).
pub const PHYS_DT: f32 = 1.0 / 60.0;
/// The net decides every this many physics ticks (20 Hz).
pub const DECISION_TICKS: u32 = 3;
/// Maximum match length in physics ticks (15 s).
pub const MAX_TICKS: u32 = 900;
/// Arena width in metres.
pub const ARENA_WIDTH: f32 = 14.0;
