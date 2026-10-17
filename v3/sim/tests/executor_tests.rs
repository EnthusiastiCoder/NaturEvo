// WS-A executor tests (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-A).
//
// INVENTED API (module `mech_sim::executor`, new file v3/sim/src/executor.rs; moveset API as in moveset_tests.rs):
//   #[derive(Clone, Debug)] pub struct Executor;
//   impl Executor {
//     pub fn new() -> Executor;
//     /// One fixed-dt tick. `req` is this tick's decision (None = no request). Requests are latched only when the
//     /// executor may start/chain/feint; otherwise ignored silently. Returns the same Action type as scripted.rs.
//     pub fn step(&mut self, me: &Robot, req: Option<(MoveId, MoveParams)>, dt: f32) -> scripted::Action;
//     pub fn move_id(&self) -> MoveId;      // current move, Idle when none
//     pub fn phase(&self) -> Phase;
//     pub fn progress(&self) -> f32;        // 0..=1 inside the current phase
//     pub fn committed(&self) -> bool;      // true from Active start until the move ends (Windup is still cancellable)
//     pub fn whiffed(&self) -> bool;        // Active ended without notify_contact(); stays set through Recovery
//     pub fn notify_contact(&mut self);     // caller reports the move touched the opponent during Active
//     pub fn energy_spent(&self) -> f32;    // cumulative energy cost charged by the executor (move + feint 8 + drag 3 + chain 5 + jerk)
//   }
// Semantics under test: start only when Idle (or chain window), robot.state()==Idle, move parts not disabled
// (moves::required joints via moveset::spec), robot.energy >= spec.energy. Feint request during Windup before 60% of
// the windup cancels it (phase leaves Windup, costs 8 energy); after 60% it is ignored. Active is never cancelled.
// Chain: a new attack request is accepted only in the last 25% of Recovery. Feint is a request-only move: it never
// becomes the move_id of a fresh start from Idle with nothing to cancel (ignored).
use mech_sim::config::PHYS_DT;
use mech_sim::executor::Executor;
use mech_sim::moveset::*;
use mech_sim::robot::{Joint, Part, Robot, Role, State, DT};

mod common;
use common::standing;

fn mid() -> MoveParams { MoveParams { reach: 0.5, speed: 0.5, arc: 0.5, delay: 0.0 } }
fn fresh() -> (Executor, Robot) { (Executor::new(), standing(Role::Hero, 3.0, 1.0)) }

/// Step once, apply the action to the robot, return nothing.
fn tick(ex: &mut Executor, r: &mut Robot, req: Option<(MoveId, MoveParams)>) {
    let a = ex.step(r, req, DT);
    r.apply_targets(&a.targets, DT);
}
fn ticks(ex: &mut Executor, r: &mut Robot, n: usize) { for _ in 0..n { tick(ex, r, None); } }
fn run_until(ex: &mut Executor, r: &mut Robot, ph: Phase, max: usize) -> usize {
    for i in 0..max {
        if ex.phase() == ph { return i; }
        tick(ex, r, None);
    }
    panic!("never reached {:?}, now {:?}", ph, ex.phase());
}
fn tip(r: &Robot) -> [f32; 2] { r.sword_capsule().b }

struct Trace { max_reach: f32, max_speed: f32 }
fn trace(r: &mut Robot, id: MoveId, p: MoveParams) -> Trace {
    let mut ex = Executor::new();
    tick(&mut ex, r, Some((id, p)));
    let mut prev = tip(r);
    let (mut max_reach, mut max_speed) = (0.0f32, 0.0f32);
    for _ in 0..140 {
        tick(&mut ex, r, None);
        let t = tip(r);
        let sh = r.capsule(Part::UArmR).a;
        max_reach = max_reach.max(((t[0] - sh[0]).powi(2) + (t[1] - sh[1]).powi(2)).sqrt());
        max_speed = max_speed.max(((t[0] - prev[0]).powi(2) + (t[1] - prev[1]).powi(2)).sqrt() / DT);
        prev = t;
        if ex.phase() == Phase::Idle && ex.move_id() == MoveId::Idle { break; }
    }
    Trace { max_reach, max_speed }
}

#[test]
fn starts_idle_and_a_request_starts_windup() {
    let (mut ex, mut r) = fresh();
    assert_eq!((ex.move_id(), ex.phase(), ex.committed()), (MoveId::Idle, Phase::Idle, false));
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    assert_eq!(ex.move_id(), MoveId::Overhead);
    assert_eq!(ex.phase(), Phase::Windup);
    assert!(!ex.committed());
}

#[test]
fn phases_run_windup_active_recovery_idle_with_spec_durations() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    let w = run_until(&mut ex, &mut r, Phase::Active, 60);
    assert!((w as f32 * PHYS_DT - 0.35).abs() < 0.06, "windup ticks {}", w);
    assert!(ex.committed());
    let a = run_until(&mut ex, &mut r, Phase::Recovery, 60);
    assert!((a as f32 * PHYS_DT - 0.10).abs() < 0.04, "active ticks {}", a);
    let rec = run_until(&mut ex, &mut r, Phase::Idle, 80);
    assert!((rec as f32 * PHYS_DT - 0.40).abs() < 0.06, "recovery ticks {}", rec);
    assert_eq!(ex.move_id(), MoveId::Idle);
    assert!(!ex.committed());
}

#[test]
fn disabled_shoulder_r_refuses_sword_moves_but_not_guard_or_kick() {
    for id in [MoveId::Overhead, MoveId::Thrust, MoveId::Parry, MoveId::Riposte] {
        let (mut ex, mut r) = fresh();
        r.set_hp(Joint::ShoulderR.part(), 0.0);
        tick(&mut ex, &mut r, Some((id, mid())));
        assert_eq!((ex.move_id(), ex.phase()), (MoveId::Idle, Phase::Idle), "{:?} must be refused", id);
    }
    for id in [MoveId::Guard, MoveId::Kick] {
        let (mut ex, mut r) = fresh();
        r.set_hp(Joint::ShoulderR.part(), 0.0);
        tick(&mut ex, &mut r, Some((id, mid())));
        assert_eq!(ex.move_id(), id, "{:?} must still start", id);
    }
}

#[test]
fn refuses_when_staggered_fallen_or_out_of_energy() {
    let (mut ex, mut r) = fresh();
    r.add_posture(150.0);
    assert_eq!(r.state(), State::Staggered);
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    assert_eq!(ex.move_id(), MoveId::Idle);

    let (mut ex, mut r) = fresh();
    r.knock_down();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    assert_eq!(ex.move_id(), MoveId::Idle);

    let (mut ex, mut r) = fresh();
    r.energy = 0.5;
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    assert_eq!(ex.move_id(), MoveId::Idle, "energy below the move cost");
}

#[test]
fn active_cannot_be_cancelled_or_replaced() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Active, 60);
    for req in [MoveId::Feint, MoveId::Backstep, MoveId::Parry, MoveId::Thrust, MoveId::Idle] {
        tick(&mut ex, &mut r, Some((req, mid())));
        if ex.phase() == Phase::Active {
            assert_eq!(ex.move_id(), MoveId::Overhead, "{:?} replaced an Active move", req);
            assert!(ex.committed());
        }
    }
}

#[test]
fn requests_while_committed_are_ignored_through_recovery_start() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Recovery, 80);
    tick(&mut ex, &mut r, Some((MoveId::Thrust, mid())));
    assert_eq!(ex.move_id(), MoveId::Overhead);
    assert_eq!(ex.phase(), Phase::Recovery);
}

#[test]
fn feint_cancels_windup_before_60_percent_and_costs_8_energy() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    ticks(&mut ex, &mut r, 4); // about 5 of 21 windup ticks, well before 60%
    assert_eq!(ex.phase(), Phase::Windup);
    let before = ex.energy_spent();
    tick(&mut ex, &mut r, Some((MoveId::Feint, mid())));
    assert_ne!(ex.phase(), Phase::Active);
    assert_ne!(ex.phase(), Phase::Windup, "feint must leave the windup");
    assert!(!ex.whiffed(), "a feint is not a whiff");
    let cost = ex.energy_spent() - before;
    assert!((cost - 8.0).abs() < 0.75, "feint cost {}", cost);
    // And the cancelled swing never reaches Active.
    for _ in 0..80 {
        tick(&mut ex, &mut r, None);
        assert_ne!(ex.phase(), Phase::Active, "cancelled move went Active");
    }
}

#[test]
fn feint_after_60_percent_of_windup_is_ignored() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    ticks(&mut ex, &mut r, 15); // about 16 of 21 ticks, 76%
    assert_eq!(ex.phase(), Phase::Windup);
    tick(&mut ex, &mut r, Some((MoveId::Feint, mid())));
    assert_eq!(ex.move_id(), MoveId::Overhead);
    run_until(&mut ex, &mut r, Phase::Active, 30);
    assert_eq!(ex.move_id(), MoveId::Overhead);
}

#[test]
fn feint_from_idle_does_nothing() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Feint, mid())));
    assert_eq!((ex.move_id(), ex.phase()), (MoveId::Idle, Phase::Idle));
    assert!(ex.energy_spent() < 1.0);
}

#[test]
fn chain_is_allowed_only_in_the_last_quarter_of_recovery() {
    // Early in recovery: ignored.
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Recovery, 80);
    ticks(&mut ex, &mut r, 4); // about 4 of 24 recovery ticks
    tick(&mut ex, &mut r, Some((MoveId::DiagDown, mid())));
    assert_eq!(ex.move_id(), MoveId::Overhead, "early chain must be ignored");
    // Late in recovery: accepted.
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Recovery, 80);
    ticks(&mut ex, &mut r, 20); // about 20 of 24, inside the last 25%
    assert_eq!(ex.phase(), Phase::Recovery);
    tick(&mut ex, &mut r, Some((MoveId::DiagDown, mid())));
    assert_eq!(ex.move_id(), MoveId::DiagDown, "chain window must accept");
    assert_eq!(ex.phase(), Phase::Windup);
}

#[test]
fn every_move_spends_positive_energy_over_its_duration() {
    for id in MoveId::ALL.iter().copied().filter(|m| *m != MoveId::Idle && *m != MoveId::Feint) {
        let (mut ex, mut r) = fresh();
        tick(&mut ex, &mut r, Some((id, mid())));
        ticks(&mut ex, &mut r, 90);
        assert!(ex.energy_spent() > 0.0, "{:?}", id);
        assert!(ex.energy_spent() < 60.0, "{:?} absurd cost {}", id, ex.energy_spent());
    }
}

#[test]
fn whiffed_flag_set_when_active_ends_without_contact() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Active, 60);
    assert!(!ex.whiffed(), "not whiffed while still Active");
    run_until(&mut ex, &mut r, Phase::Recovery, 60);
    assert!(ex.whiffed());
}

#[test]
fn whiffed_flag_not_set_after_contact() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Active, 60);
    ex.notify_contact();
    run_until(&mut ex, &mut r, Phase::Recovery, 60);
    assert!(!ex.whiffed());
}

#[test]
fn whiffed_flag_clears_for_the_next_move() {
    let (mut ex, mut r) = fresh();
    tick(&mut ex, &mut r, Some((MoveId::Overhead, mid())));
    run_until(&mut ex, &mut r, Phase::Idle, 150);
    tick(&mut ex, &mut r, Some((MoveId::Thrust, mid())));
    assert_eq!(ex.move_id(), MoveId::Thrust);
    assert!(!ex.whiffed());
}

#[test]
fn reach_up_extends_the_tip_for_thrust_and_overhead() {
    for id in [MoveId::Thrust, MoveId::Overhead] {
        let lo = trace(&mut standing(Role::Hero, 3.0, 1.0), id, MoveParams { reach: 0.0, ..mid() });
        let hi = trace(&mut standing(Role::Hero, 3.0, 1.0), id, MoveParams { reach: 1.0, ..mid() });
        assert!(hi.max_reach > lo.max_reach + 0.05, "{:?} reach {} vs {}", id, lo.max_reach, hi.max_reach);
    }
}

#[test]
fn speed_up_raises_peak_tip_speed() {
    for id in [MoveId::Thrust, MoveId::Overhead] {
        let slow = trace(&mut standing(Role::Hero, 3.0, 1.0), id, MoveParams { speed: 0.2, ..mid() });
        let fast = trace(&mut standing(Role::Hero, 3.0, 1.0), id, MoveParams { speed: 1.0, ..mid() });
        assert!(fast.max_speed > slow.max_speed * 1.1, "{:?} {} vs {}", id, slow.max_speed, fast.max_speed);
    }
}
