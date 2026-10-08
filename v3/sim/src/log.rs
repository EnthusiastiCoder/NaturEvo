//! Fight log (plan section 6): hand-rolled JSON consumed by the viewer. Frames every 2 physics ticks,
//! capsule endpoints in integer centimetres, events with point, direction and impact.
use crate::duel::{Match, Outcome};
use crate::robot::{Capsule, Part, Robot, State};
use std::fmt::Write;
