//! Imitation: distil a scripted style teacher into a net (supervised, from the net's own sensed inputs).
//! The loss is a softmax cross-entropy over the 14 move outputs plus squared error on the four move params (only when
//! a move is asked for) and on the footwork drive. The recurrent context is carried through each fight in order and
//! treated as a constant input (truncated backprop). Later rounds (DAgger) let the current student fight while the
//! teacher watches and labels the states the student actually reaches.
use crate::arena::Arena;
use crate::duel::{Brain, Match, Sample};
use crate::genome::Genome;
use crate::net::{fast_tanh, N_CTX, N_HID, N_OUT, N_PARAMS};
use crate::moveset::N_MOVES;
use crate::rng::Rng;
use crate::robot::Role;
use crate::sense::N_IN;
use crate::stylebot::StyleId;

/// Softmax temperature on the tanh move outputs.
const K_LOGIT: f32 = 4.0;
/// Extra weight on samples that ask for a move (Idle dominates the data).
const W_MOVE: f32 = 2.5;
const W_PARAM: f32 = 1.0;
const W_DRIVE: f32 = 1.0;
/// Longest fight sequence kept (decision steps).
const MAX_SEQ: usize = 1200;
