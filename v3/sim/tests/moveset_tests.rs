// WS-A moveset data tests (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-A).
//
// INVENTED API (module `mech_sim::moveset`, new file v3/sim/src/moveset.rs):
//   pub const N_MOVES: usize = 14;
//   #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum MoveId { Idle, Overhead, DiagDown, DiagUp, Sweep, Thrust,
//        LowCut, Feint, Parry, Guard, Riposte, ShieldBash, Kick, Backstep }   // discriminants 0..=13, fixed order
//   impl MoveId { pub const ALL: [MoveId; 14]; pub fn index(self) -> usize; pub fn name(self) -> &'static str; }
//   #[derive(Clone, Copy, Debug, PartialEq)] pub struct MoveParams { pub reach: f32, pub speed: f32, pub arc: f32, pub delay: f32 } // each 0..=1
//   #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Phase { Idle, Windup, Active, Recovery }
//   pub struct MoveSpec { pub windup: f32, pub active: f32, pub recovery: f32,   // seconds at speed 0.5, delay 0
//                         pub required: &'static [Joint], pub energy: f32 }
//   pub fn spec(id: MoveId) -> &'static MoveSpec;
//   pub fn phase_durations(id: MoveId, p: &MoveParams) -> [f32; 3];   // [windup (incl. delay), active, recovery] seconds
//   pub fn min_jerk(p: f32) -> f32;                                   // 10p^3 - 15p^4 + 6p^5
//   pub fn pose_at(id: MoveId, p: &MoveParams, phase: Phase, t: f32) -> [f32; 12];  // normalised targets in [-1,1], t in 0..=1
use mech_sim::moveset::*;
use mech_sim::robot::Joint;

fn mid() -> MoveParams { MoveParams { reach: 0.5, speed: 0.5, arc: 0.5, delay: 0.0 } }
fn attacks() -> Vec<MoveId> { MoveId::ALL.iter().copied().filter(|m| *m != MoveId::Idle).collect() }
fn jump(a: &[f32; 12], b: &[f32; 12]) -> f32 { (0..12).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max) }
fn total(id: MoveId, p: &MoveParams) -> f32 { phase_durations(id, p).iter().sum() }

#[test]
fn move_id_order_and_count_are_fixed() {
    assert_eq!(N_MOVES, 14);
    assert_eq!(MoveId::ALL.len(), 14);
    let order = [MoveId::Idle, MoveId::Overhead, MoveId::DiagDown, MoveId::DiagUp, MoveId::Sweep, MoveId::Thrust, MoveId::LowCut,
        MoveId::Feint, MoveId::Parry, MoveId::Guard, MoveId::Riposte, MoveId::ShieldBash, MoveId::Kick, MoveId::Backstep];
    for (i, m) in order.iter().enumerate() {
        assert_eq!(MoveId::ALL[i], *m);
        assert_eq!(m.index(), i);
    }
    assert_eq!(MoveId::Backstep as usize, 13);
}

#[test]
fn every_non_idle_move_has_required_joints_and_positive_phases() {
    for m in attacks() {
        let s = spec(m);
        assert!(!s.required.is_empty(), "{:?} required joints", m);
        assert!(s.windup > 0.0 && s.active > 0.0 && s.recovery > 0.0, "{:?} W/A/R {} {} {}", m, s.windup, s.active, s.recovery);
        assert!(s.energy > 0.0, "{:?} energy", m);
        assert!(!m.name().is_empty());
    }
}

#[test]
fn spec_times_match_the_plan_table() {
    let table = [(MoveId::Overhead, 0.35, 0.10, 0.40), (MoveId::DiagDown, 0.30, 0.10, 0.35), (MoveId::DiagUp, 0.25, 0.10, 0.30),
        (MoveId::Sweep, 0.30, 0.15, 0.45), (MoveId::Thrust, 0.20, 0.08, 0.35), (MoveId::LowCut, 0.30, 0.12, 0.40),
        (MoveId::Parry, 0.05, 0.15, 0.30), (MoveId::ShieldBash, 0.25, 0.10, 0.35), (MoveId::Kick, 0.15, 0.08, 0.30),
        (MoveId::Backstep, 0.20, 0.10, 0.25)];
    for (m, w, a, r) in table {
        let s = spec(m);
        assert!((s.windup - w).abs() < 1e-4 && (s.active - a).abs() < 1e-4 && (s.recovery - r).abs() < 1e-4, "{:?}", m);
    }
}

#[test]
fn required_joints_follow_the_plan() {
    let has = |m: MoveId, js: &[Joint]| {
        let s = spec(m);
        for j in js { assert!(s.required.contains(j), "{:?} needs {:?}", m, j); }
    };
    has(MoveId::Overhead, &[Joint::HipR, Joint::Waist, Joint::ShoulderR, Joint::ElbowR]);
    has(MoveId::Thrust, &[Joint::HipR, Joint::KneeR, Joint::Waist, Joint::ShoulderR, Joint::ElbowR]);
    has(MoveId::Parry, &[Joint::ShoulderR, Joint::ElbowR, Joint::Waist]);
    has(MoveId::Guard, &[Joint::ShoulderL, Joint::ElbowL]);
    has(MoveId::ShieldBash, &[Joint::Waist, Joint::ShoulderL, Joint::ElbowL]);
    has(MoveId::Kick, &[Joint::HipR, Joint::KneeR, Joint::AnkleR]);
    has(MoveId::Backstep, &[Joint::HipL, Joint::HipR, Joint::KneeL, Joint::KneeR, Joint::AnkleL, Joint::AnkleR]);
    assert!(!spec(MoveId::Guard).required.contains(&Joint::ShoulderR));
    assert!(!spec(MoveId::Overhead).required.contains(&Joint::ShoulderL));
}

#[test]
fn pose_is_finite_and_within_unit_range_everywhere() {
    for m in MoveId::ALL {
        for &(r, s, a, d) in &[(0.0, 0.0, 0.0, 0.0), (0.5, 0.5, 0.5, 0.0), (1.0, 1.0, 1.0, 1.0), (0.0, 1.0, 0.0, 1.0), (1.0, 0.0, 1.0, 0.5)] {
            let p = MoveParams { reach: r, speed: s, arc: a, delay: d };
            for ph in [Phase::Idle, Phase::Windup, Phase::Active, Phase::Recovery] {
                for k in 0..=20 {
                    let pose = pose_at(m, &p, ph, k as f32 / 20.0);
                    for v in pose { assert!(v.is_finite() && (-1.0..=1.0).contains(&v), "{:?} {:?} {} -> {}", m, ph, k, v); }
                }
            }
        }
    }
}

#[test]
fn pose_is_continuous_across_phase_boundaries_at_60hz() {
    for m in attacks() {
        let p = mid();
        let d = phase_durations(m, &p);
        let mut seq: Vec<[f32; 12]> = Vec::new();
        for (ph, dur) in [(Phase::Windup, d[0]), (Phase::Active, d[1]), (Phase::Recovery, d[2])] {
            let n = ((dur * 60.0).round() as usize).max(1);
            for k in 0..=n { seq.push(pose_at(m, &p, ph, k as f32 / n as f32)); }
        }
        seq.push(pose_at(MoveId::Idle, &p, Phase::Idle, 0.0));
        for w in seq.windows(2) {
            assert!(jump(&w[0], &w[1]) < 0.35, "{:?} jump {}", m, jump(&w[0], &w[1]));
        }
    }
}

#[test]
fn keyframes_are_shared_between_adjacent_phases() {
    for m in attacks() {
        let p = mid();
        assert!(jump(&pose_at(m, &p, Phase::Windup, 1.0), &pose_at(m, &p, Phase::Active, 0.0)) < 1e-4, "{:?} windup->active", m);
        assert!(jump(&pose_at(m, &p, Phase::Active, 1.0), &pose_at(m, &p, Phase::Recovery, 0.0)) < 1e-4, "{:?} active->recovery", m);
    }
}

#[test]
fn min_jerk_endpoints_exact_and_zero_velocity() {
    assert_eq!(min_jerk(0.0), 0.0);
    assert!((min_jerk(1.0) - 1.0).abs() < 1e-6);
    assert!((min_jerk(0.5) - 0.5).abs() < 1e-6);
    let e = 1e-3;
    assert!(min_jerk(e) / e < 1e-4, "start velocity");
    assert!((1.0 - min_jerk(1.0 - e)) / e < 1e-4, "end velocity");
    let mut prev = 0.0;
    for k in 1..=100 {
        let v = min_jerk(k as f32 / 100.0);
        assert!(v >= prev);
        prev = v;
    }
    let t: f32 = 0.3;
    assert!((min_jerk(t) - (10.0 * t.powi(3) - 15.0 * t.powi(4) + 6.0 * t.powi(5))).abs() < 1e-6);
}

#[test]
fn windup_starts_and_recovery_ends_with_near_zero_velocity() {
    for m in attacks() {
        let p = mid();
        let a = pose_at(m, &p, Phase::Windup, 0.0);
        let b = pose_at(m, &p, Phase::Windup, 0.01);
        assert!(jump(&a, &b) < 0.01, "{:?} windup start speed", m);
        let c = pose_at(m, &p, Phase::Recovery, 0.99);
        let d = pose_at(m, &p, Phase::Recovery, 1.0);
        assert!(jump(&c, &d) < 0.01, "{:?} recovery end speed", m);
    }
}
