//! Scripted style teachers: three archetypes built from the existing move vocabulary, each a function of what a
//! fighter can sense (distance, own and opponent move phase, opening flags), so a net can imitate them.
//! They fight through the same Executor as the net and obey the same rules as every fighter.
//! Press: advances, chains attacks from close range, rarely defends. Counter: holds distance, defends against
//! every telegraph, then ripostes and punishes. Duelist: keeps its optimal spacing, strikes singly with feints
//! and drag, steps out of reach of incoming swings.
use crate::moveset::{MoveId, MoveParams, Phase};
use crate::robot::{Robot, State};
use crate::rng::Rng;
use crate::sense::MoveInfo;
use crate::skillbot::{is_attack, Request};
