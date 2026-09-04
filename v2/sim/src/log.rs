//! Fight log: one frame per tick, serialised to JSON by hand (no dependencies).
use crate::fighter::{Action, CombatState};
use std::fmt::Write;
