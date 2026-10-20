//! Scripted opponents that fight through the same Executor as the net: three tiers for a curriculum.
//! Tier 1 Brawler (attacks only), tier 2 Reactive (reads the telegraph, guards or parries, punishes Open),
//! tier 3 Duellist (adds feints, drag, backstep, shield bash, riposte). Deterministic from the seed.
use crate::moveset::{MoveId, MoveParams, Phase};
use crate::rng::Rng;
use crate::robot::{Robot, State};
use crate::sense::MoveInfo;

pub type Request = Option<(MoveId, MoveParams)>;

const SLASHES: [MoveId; 6] = [MoveId::Overhead, MoveId::DiagDown, MoveId::Sweep, MoveId::Thrust, MoveId::DiagUp, MoveId::LowCut];
/// Torso distance band a bot swings from, metres.
const RANGE_LO: f32 = 1.9;
const RANGE_HI: f32 = 2.2;
const CLOSE: f32 = 1.35;
/// Retreat briefly after a passive opponent staggers, to avoid its recoil sweep.
const FLAIL_TIME: f32 = 1.0;
const FLAIL_DIST: f32 = 3.5;
const FLAIL_IDLE: f32 = 2.0;
const LOW_MOVE_P: f32 = 0.1;
