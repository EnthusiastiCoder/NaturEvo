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

#[test]
fn decode_requires_the_margin_over_idle() {
    let below = out_with(|o| { o[0] = 0.50; o[1] = 0.50 + MOVE_MARGIN * 0.5; });
    assert_eq!(decode(&below).mv, MoveId::Idle);
    let equal = out_with(|o| { o[0] = 0.5; o[4] = 0.5; });
    assert_eq!(decode(&equal).mv, MoveId::Idle);
    let above = out_with(|o| { o[0] = 0.50; o[1] = 0.50 + MOVE_MARGIN * 2.0; });
    assert_eq!(decode(&above).mv, MoveId::Overhead);
    approx(MOVE_MARGIN, 0.05);
}

#[test]
fn decode_idle_wins_when_it_is_the_max() {
    let o = out_with(|o| { o[0] = 0.9; o[6] = 0.2; });
    assert_eq!(decode(&o).mv, MoveId::Idle);
}

fn approx(a: f32, b: f32) { assert!((a - b).abs() < 1e-5, "{} vs {}", a, b); }

#[test]
fn decode_maps_params_to_unit_range_and_clamps_drive() {
    let d = decode(&out_with(|o| { o[14] = -1.0; o[15] = 0.0; o[16] = 1.0; o[17] = 0.5; o[18] = 0.25; }));
    let p: MoveParams = d.params;
    approx(p.reach, 0.0);
    approx(p.speed, 0.5);
    approx(p.arc, 1.0);
    approx(p.delay, 0.75);
    approx(d.drive, 0.25);
    let e = decode(&out_with(|o| { o[14] = -3.0; o[15] = 3.0; o[18] = 2.0; }));
    assert_eq!((e.params.reach, e.params.speed), (0.0, 1.0));
    assert_eq!(e.drive, 1.0);
}

#[test]
fn decode_is_pure() {
    let o = out_with(|o| { o[9] = 0.9; o[14] = 0.3; });
    let (a, b) = (decode(&o), decode(&o));
    assert_eq!((a.mv, a.drive), (b.mv, b.drive));
    assert_eq!(a.params, b.params);
}

fn pair() -> (Robot, Robot, Arena) { (standing(Role::Hero, 4.0, 1.0), standing(Role::Monster, 7.0, -1.0), Arena::flat(14.0)) }
fn info(id: MoveId, phase: Phase) -> MoveInfo { MoveInfo { id, phase, progress: 0.4, ..Default::default() } }

#[test]
fn old_observe_equals_observe_moves_with_idle_infos() {
    let (a, b, ar) = pair();
    let d = MoveInfo::default();
    assert_eq!(observe(&a, &b, &ar)[..], observe_moves(&a, &b, &ar, &d, &d)[..]);
}

#[test]
fn input_ranges_for_move_and_phase_one_hots_exist_and_do_not_overlap() {
    for (r1, r2) in [(&OWN_MOVE, &OWN_PHASE), (&OWN_MOVE, &OPP_MOVE), (&OWN_MOVE, &OPP_PHASE), (&OWN_PHASE, &OPP_MOVE), (&OWN_PHASE, &OPP_PHASE), (&OPP_MOVE, &OPP_PHASE)] {
        assert!(r1.end <= r2.start || r2.end <= r1.start, "{:?} overlaps {:?}", r1, r2);
    }
    assert_eq!((OWN_MOVE.len(), OWN_PHASE.len(), OPP_MOVE.len(), OPP_PHASE.len()), (14, 4, 14, 4));
    assert!(OWN_MOVE.end <= N_IN && OPP_MOVE.end <= N_IN && OWN_PHASE.end <= N_IN && OPP_PHASE.end <= N_IN);
}

#[test]
fn own_move_and_phase_are_one_hot() {
    let (a, b, ar) = pair();
    let x = observe_moves(&a, &b, &ar, &info(MoveId::Overhead, Phase::Active), &MoveInfo::default());
    for i in 0..14 { assert_eq!(x[OWN_MOVE.start + i], if i == MoveId::Overhead as usize { 1.0 } else { 0.0 }, "own move {}", i); }
    for i in 0..4 { assert_eq!(x[OWN_PHASE.start + i], if i == Phase::Active as usize { 1.0 } else { 0.0 }, "own phase {}", i); }
}

#[test]
fn opponent_move_and_phase_mirror_correctly() {
    let (a, b, ar) = pair();
    let ia = info(MoveId::Overhead, Phase::Windup);
    let ib = info(MoveId::Thrust, Phase::Recovery);
    let xa = observe_moves(&a, &b, &ar, &ia, &ib);
    let xb = observe_moves(&b, &a, &ar, &ib, &ia);
    // What A sees as the opponent is what B sees as own, and vice versa.
    assert_eq!(xa[OPP_MOVE.clone()], xb[OWN_MOVE.clone()]);
    assert_eq!(xa[OPP_PHASE.clone()], xb[OWN_PHASE.clone()]);
    assert_eq!(xb[OPP_MOVE.clone()], xa[OWN_MOVE.clone()]);
    assert_eq!(xa[OPP_MOVE.start + MoveId::Thrust as usize], 1.0);
    assert_eq!(xa[OPP_PHASE.start + Phase::Recovery as usize], 1.0);
}

#[test]
fn opponent_move_inputs_are_zero_when_own_head_is_disabled() {
    let (mut a, b, ar) = pair();
    a.set_hp(Part::Head, 0.0);
    let x = observe_moves(&a, &b, &ar, &info(MoveId::Parry, Phase::Idle), &info(MoveId::Overhead, Phase::Active));
    assert!(x[OPP_MOVE.clone()].iter().all(|v| *v == 0.0), "opp move not blanked");
    assert!(x[OPP_PHASE.clone()].iter().all(|v| *v == 0.0), "opp phase not blanked");
    // Own state is still known to a blinded fighter.
    assert_eq!(x[OWN_MOVE.start + MoveId::Parry as usize], 1.0);
}

#[test]
fn observations_are_finite_and_in_range_under_fuzz() {
    let mut rng = Rng::new(99);
    let ar = Arena::flat(14.0);
    for _ in 0..200 {
        let mut a = standing(Role::Hero, rng.range(1.0, 6.0), 1.0);
        let mut b = standing(Role::Monster, rng.range(7.0, 13.0), -1.0);
        for p in Part::ALL {
            if rng.f32() < 0.2 { a.set_hp(p, a.max_hp(p) * rng.f32()); }
            if rng.f32() < 0.2 { b.set_hp(p, b.max_hp(p) * rng.f32()); }
        }
        for _ in 0..5 {
            let mut t = STAND_TARGETS;
            for v in t.iter_mut() { *v = rng.range(-1.0, 1.0); }
            a.apply_targets(&t, DT);
            b.apply_targets(&t, DT);
        }
        let rnd = |rng: &mut Rng| MoveInfo {
            id: MoveId::ALL[(rng.next_u64() % 14) as usize],
            phase: [Phase::Idle, Phase::Windup, Phase::Active, Phase::Recovery][(rng.next_u64() % 4) as usize],
            progress: rng.f32(), committed: rng.f32() < 0.5, open: rng.f32() < 0.5,
            riposte_remaining: rng.range(0.0, 0.6), guard_broken_remaining: rng.range(0.0, 0.4),
            last_cost: rng.range(0.0, 30.0), recovery_remaining: rng.range(0.0, 0.5),
        };
        let (ma, mb) = (rnd(&mut rng), rnd(&mut rng));
        for x in [observe_moves(&a, &b, &ar, &ma, &mb), observe_moves(&b, &a, &ar, &mb, &ma)] {
            assert_eq!(x.len(), N_IN);
            assert!(x.iter().all(|v| v.is_finite() && (-3.0..=3.0).contains(v)));
        }
    }
}
