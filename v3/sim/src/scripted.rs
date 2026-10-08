//! Scripted opponents: a static dummy and an attacker that approaches and swings at random
//! intervals (slash high, slash low, thrust). Deterministic from the match seed.
use crate::robot::{Joint, Robot, STAND_TARGETS};
use crate::rng::Rng;
