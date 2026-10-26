//! Measurement of what trained nets do in capped fights (diagnostics only, never read by fitness).
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::moveset::MoveId;
use crate::skillbot::is_attack;
use std::path::Path;
