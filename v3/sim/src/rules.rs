//! Game-rule lethality for the sword vocabulary: Open windows, riposte, guard break, heavy block, backstep counter.
//! Pure data and functions: no RNG, no fitness or net access.
use crate::moveset::MoveId;
use crate::robot::Robot;

pub const OPEN_MULT: f32 = 1.30;
pub const OPEN_POSTURE_MULT: f32 = 1.5;
pub const COUNTER_HIT_MULT: f32 = 1.15;
pub const MULT_CAP: f32 = 1.75;
pub const RIPOSTE_WINDOW: f32 = 0.6;
pub const RIPOSTE_MULT: f32 = 1.5;
pub const RIPOSTE_WINDUP_FAST: f32 = 0.15;
pub const RIPOSTE_WINDUP_SLOW: f32 = 0.30;
pub const PARRY_MISS_RECOVERY: f32 = 0.30;
pub const PARRY_MISS_OPEN_MULT: f32 = 1.2;
pub const SHIELD_BASH_POSTURE: f32 = 35.0;
pub const GUARD_BREAK_TIME: f32 = 0.4;
pub const GUARD_BREAK_OPEN_MULT: f32 = 1.25;
pub const HEAVY_BLOCK_POSTURE_MULT: f32 = 1.4;
pub const BACKSTEP_COUNTER_POSTURE: f32 = 30.0;
pub const BACKSTEP_COUNTER_OPEN: f32 = 0.3;
pub const FEINT_COST: f32 = 8.0;
pub const DRAG_COST: f32 = 3.0;
pub const CHAIN_COST: f32 = 5.0;
/// Multiplier on exposed (staggered) victims.
const EXPOSED_MULT: f32 = 1.5;
