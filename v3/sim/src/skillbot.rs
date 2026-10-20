//! Scripted opponents that fight through the same Executor as the net: three tiers for a curriculum.
//! Tier 1 Brawler (attacks only), tier 2 Reactive (reads the telegraph, guards or parries, punishes Open),
//! tier 3 Duellist (adds feints, drag, backstep, shield bash, riposte). Deterministic from the seed.
use crate::moveset::{MoveId, MoveParams, Phase};
use crate::rng::Rng;
use crate::robot::{Robot, State};
use crate::sense::MoveInfo;

pub type Request = Option<(MoveId, MoveParams)>;
