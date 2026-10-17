// WS-C net IO and sensing tests (plan 2026-10-23_v3-sword-vocabulary.md, "Net IO and sensing", Test cases WS-C).
//
// INVENTED API:
//   mech_sim::net:
//     pub const N_OUT: usize = 19;                     // 14 move logits, 4 params (reach, speed, arc, delay), 1 drive
//     pub const MOVE_MARGIN: f32 = 0.05;
//     pub struct Decoded { pub mv: MoveId, pub params: MoveParams, pub drive: f32 }
//     pub fn decode(out: &[f32; N_OUT]) -> Decoded;
//       outputs are post-tanh in [-1,1]. mv = argmax of out[0..14], ties to the LOWEST index, and a non-Idle move is chosen
//       only if its logit exceeds out[0] (Idle) by MOVE_MARGIN, otherwise Idle. params[i] = (out[14+i] + 1) / 2 clamped to 0..=1.
//       drive = out[18] clamped to -1..=1. (Whether the move is currently allowed is the Executor's business, not decode's.)
//   mech_sim::sense:
//     pub const N_IN: usize                           // symbolic, grows to about 140; tests only require 90..=160
//     pub const OWN_MOVE: Range<usize> (14), OWN_PHASE: Range<usize> (4), OPP_MOVE: Range<usize> (14), OPP_PHASE: Range<usize> (4)
//       one-hot own/opponent move id and phase indices inside the observation vector; ranges must not overlap each other.
//     #[derive(Clone, Copy, Debug, Default)] pub struct MoveInfo { pub id: MoveId (Default = Idle), pub phase: Phase (Default = Idle),
//         pub progress: f32, pub committed: bool, pub open: bool, pub riposte_remaining: f32, pub guard_broken_remaining: f32,
//         pub last_cost: f32, pub recovery_remaining: f32 }
//     pub fn observe_moves(me: &Robot, opp: &Robot, arena: &Arena, mine: &MoveInfo, theirs: &MoveInfo) -> [f32; N_IN];
//     The existing `observe(me, opp, arena)` keeps its signature and equals observe_moves with two default (Idle) infos.
use mech_sim::arena::Arena;
use mech_sim::moveset::{MoveId, MoveParams, Phase};
use mech_sim::net::{decode, MOVE_MARGIN, N_OUT, N_PARAMS};
use mech_sim::rng::Rng;
use mech_sim::robot::{Part, Robot, Role, STAND_TARGETS, DT};
use mech_sim::sense::*;

mod common;
use common::standing;

fn out_with(f: impl Fn(&mut [f32; N_OUT])) -> [f32; N_OUT] {
    let mut o = [-1.0f32; N_OUT];
    o[14..].iter_mut().for_each(|v| *v = 0.0);
    f(&mut o);
    o
}

#[test]
fn n_out_is_19_and_sizes_are_sane() {
    assert_eq!(N_OUT, 19);
    assert!((90..=160).contains(&N_IN), "N_IN {}", N_IN);
    assert!((4000..=7000).contains(&N_PARAMS), "N_PARAMS {}", N_PARAMS);
}

#[test]
fn decode_picks_the_max_logit() {
    let o = out_with(|o| { o[0] = 0.0; o[5] = 0.8; o[2] = 0.3; });
    assert_eq!(decode(&o).mv, MoveId::Thrust);
}

#[test]
fn decode_ties_go_to_the_lowest_index() {
    let o = out_with(|o| { o[0] = 0.0; o[3] = 0.7; o[7] = 0.7; o[11] = 0.7; });
    assert_eq!(decode(&o).mv, MoveId::DiagUp);
}
