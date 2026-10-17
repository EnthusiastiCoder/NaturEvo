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
