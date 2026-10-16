// WS-A moveset data tests (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-A).
//
// INVENTED API (module `mech_sim::moveset`, new file v3/sim/src/moveset.rs):
//   pub const N_MOVES: usize = 14;
//   #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum MoveId { Idle, Overhead, DiagDown, DiagUp, Sweep, Thrust,
//        LowCut, Feint, Parry, Guard, Riposte, ShieldBash, Kick, Backstep }   // discriminants 0..=13, fixed order
//   impl MoveId { pub const ALL: [MoveId; 14]; pub fn index(self) -> usize; pub fn name(self) -> &'static str; }
//   #[derive(Clone, Copy, Debug, PartialEq)] pub struct MoveParams { pub reach: f32, pub speed: f32, pub arc: f32, pub delay: f32 } // each 0..=1
//   #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Phase { Idle, Windup, Active, Recovery }
//   pub struct MoveSpec { pub windup: f32, pub active: f32, pub recovery: f32,   // seconds at speed 0.5, delay 0
//                         pub required: &'static [Joint], pub energy: f32 }
//   pub fn spec(id: MoveId) -> &'static MoveSpec;
//   pub fn phase_durations(id: MoveId, p: &MoveParams) -> [f32; 3];   // [windup (incl. delay), active, recovery] seconds
//   pub fn min_jerk(p: f32) -> f32;                                   // 10p^3 - 15p^4 + 6p^5
//   pub fn pose_at(id: MoveId, p: &MoveParams, phase: Phase, t: f32) -> [f32; 12];  // normalised targets in [-1,1], t in 0..=1
use mech_sim::moveset::*;
use mech_sim::robot::Joint;

fn mid() -> MoveParams { MoveParams { reach: 0.5, speed: 0.5, arc: 0.5, delay: 0.0 } }
fn attacks() -> Vec<MoveId> { MoveId::ALL.iter().copied().filter(|m| *m != MoveId::Idle).collect() }
fn jump(a: &[f32; 12], b: &[f32; 12]) -> f32 { (0..12).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max) }
fn total(id: MoveId, p: &MoveParams) -> f32 { phase_durations(id, p).iter().sum() }
