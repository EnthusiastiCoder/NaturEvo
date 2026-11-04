//! Blades are solid against each other: no tick ends with the two blades overlapping, and no blade passes through the
//! other between ticks (swept test), whatever the fighters do. The same for both fighters.
use mech_sim::arena::Arena;
use mech_sim::combat::seg_seg;
use mech_sim::duel::{Brain, Match};
use mech_sim::moveset::MoveId;
use mech_sim::robot::Capsule;
use mech_sim::stylebot::StyleId;
