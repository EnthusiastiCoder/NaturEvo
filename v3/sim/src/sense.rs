//! Sensing: the network input vector, in the fighter's own facing frame (forward = +x).
//! The opponent is always visible (no FOV). A damaged head adds deterministic noise to opponent
//! inputs; a disabled head blinds the fighter (opponent inputs are zero).
use crate::arena::Arena;
use crate::config::PHYS_DT;
use crate::moveset::{MoveId, Phase};
use crate::rng::Rng;
use crate::robot::{Capsule, Joint, Part, Robot, State};
use std::ops::Range;

/// Number of normalised network inputs.
pub const BASE_IN: usize = 104;
/// Own move one-hot (14), phase one-hot (4), then progress, committed, open, riposte window, guard-broken, last cost (6).
pub const OWN_MOVE: Range<usize> = 104..118;
pub const OWN_PHASE: Range<usize> = 118..122;
/// Opponent move one-hot (14), phase one-hot (4), then progress, open, recovery remaining (3): the telegraph.
pub const OPP_MOVE: Range<usize> = 128..142;
pub const OPP_PHASE: Range<usize> = 142..146;
pub const N_IN: usize = 149;

/// What the executor and rules know about one fighter's current move.
#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInfo {
    pub id: MoveId,
    pub phase: Phase,
    pub progress: f32,
    pub committed: bool,
    pub open: bool,
    pub riposte_remaining: f32,
    pub guard_broken_remaining: f32,
    pub last_cost: f32,
    pub recovery_remaining: f32,
}

impl Default for MoveId {
    fn default() -> Self {
        MoveId::Idle
    }
}
impl Default for Phase {
    fn default() -> Self {
        Phase::Idle
    }
}

/// Opponent key points (8 points, x then y each) in own frame, relative to own torso centre.
pub const OPP_POINTS: Range<usize> = 64..80;
/// Opponent sword tip velocity (2), speed toward me, time to contact, threat height.
pub const OPP_SWORD: Range<usize> = 80..85;
/// Opponent part hp fractions (13).
pub const OPP_HP: Range<usize> = 85..98;

const POS_SCALE: f32 = 4.0;
const VEL_SCALE: f32 = 10.0;

fn mid(c: &Capsule) -> [f32; 2] {
    [(c.a[0] + c.b[0]) * 0.5, (c.a[1] + c.b[1]) * 0.5]
}

fn norm_angle(r: &Robot, j: Joint) -> f32 {
    let (lo, hi) = j.limits();
    ((r.joint_angle(j) - lo) / (hi - lo) * 2.0 - 1.0).clamp(-1.0, 1.0)
}

struct W {
    x: [f32; N_IN],
    i: usize,
}

impl W {
    fn put(&mut self, v: f32) {
        if self.i < N_IN {
            let v = if v.is_finite() { v } else { 0.0 };
            self.x[self.i] = v.clamp(-3.0, 3.0);
        }
        self.i += 1;
    }
}

/// Distance ahead of and behind the fighter to the nearest obstacle edge or wall.
fn wall_distances(me: &Robot, arena: &Arena) -> (f32, f32) {
    let (mut right, mut left) = (arena.width - me.x, me.x);
    for o in &arena.obstacles {
        let lo = o.x - o.w * 0.5;
        let hi = o.x + o.w * 0.5;
        if lo >= me.x {
            right = right.min(lo - me.x);
        } else if hi <= me.x {
            left = left.min(me.x - hi);
        }
    }
    if me.facing >= 0.0 {
        (right, left)
    } else {
        (left, right)
    }
}

/// True when an obstacle stands between the two torso centres.
fn line_of_sight_blocked(a: f32, b: f32, arena: &Arena) -> bool {
    let (lo, hi) = (a.min(b), a.max(b));
    arena.obstacles.iter().any(|o| o.x > lo && o.x < hi)
}

pub fn observe(me: &Robot, opp: &Robot, arena: &Arena) -> [f32; N_IN] {
    let d = MoveInfo::default();
    observe_moves(me, opp, arena, &d, &d)
}

pub fn observe_moves(me: &Robot, opp: &Robot, arena: &Arena, mine: &MoveInfo, theirs: &MoveInfo) -> [f32; N_IN] {
    let mut w = W { x: [0.0; N_IN], i: 0 };
    let f = if me.facing >= 0.0 { 1.0 } else { -1.0 };
    let o = mid(&me.capsule(Part::Torso));
    let rel = |p: [f32; 2]| -> [f32; 2] { [(p[0] - o[0]) * f / POS_SCALE, (p[1] - o[1]) / POS_SCALE] };
    let relv = |v: [f32; 2]| -> [f32; 2] { [v[0] * f / VEL_SCALE, v[1] / VEL_SCALE] };

    // Own joints (24).
    for j in Joint::ALL {
        w.put(norm_angle(me, j));
    }
    for j in Joint::ALL {
        w.put(me.joint_vel[j as usize] / j.max_speed());
    }
    // Own part hp (13) and disabled flags (13).
    for p in Part::ALL {
        w.put(me.hp_frac(p));
    }
    // Scalars (6).
    w.put(me.energy / 100.0);
    w.put(me.posture / 100.0);
    w.put(me.balance_margin());
    let st = me.state();
    w.put(if st == State::Fallen || st == State::Ko { 0.0 } else { 1.0 });
    w.put(if st == State::Fallen { 1.0 } else { 0.0 });
    w.put(if st == State::Staggered { 1.0 } else { 0.0 });
    // Own sword tip position and velocity (4), shield centre (2).
    let sw = me.sword_capsule();
    let swp = me.sword_prev_capsule();
    let p = rel(sw.b);
    w.put(p[0]);
    w.put(p[1]);
    let v = relv([(sw.b[0] - swp.b[0]) / PHYS_DT, (sw.b[1] - swp.b[1]) / PHYS_DT]);
    w.put(v[0]);
    w.put(v[1]);
    let sh = me.shield_capsule();
    let p = rel(mid(&sh));
    w.put(p[0]);
    w.put(p[1]);
    // Own disabled flags (13).
    for pt in Part::ALL {
        w.put(if me.is_disabled(pt) { 1.0 } else { 0.0 });
    }
    // Sword angular state (2).
    w.put(me.sword_ang_vel / 20.0);
    w.put(me.sword_moving_since);
    debug_assert_eq!(w.i, OPP_POINTS.start);

    // Opponent.
    let head = me.hp_frac(Part::Head);
    let blind = me.is_disabled(Part::Head);
    let noise = if blind { 0.0 } else if head < 0.5 { (1.0 - head * 2.0) * 0.3 } else { 0.0 };
    let mut rng = Rng::new(me.hp(Part::Head).to_bits() as u64 ^ ((opp.x * 1000.0) as i64 as u64).rotate_left(17));
    let mut opp_in = [0.0f32; 34];
    {
        let (fa, fb) = (opp.capsule(Part::FootL), opp.capsule(Part::FootR));
        let (ma, mb) = (mid(&fa), mid(&fb));
        let (front, rear) = if (ma[0] * opp.facing) >= (mb[0] * opp.facing) { (ma, mb) } else { (mb, ma) };
        let pts = [
            mid(&opp.capsule(Part::Head)),
            mid(&opp.capsule(Part::Torso)),
            mid(&opp.capsule(Part::Pelvis)),
            opp.sword_capsule().b,
            opp.sword_capsule().a,
            mid(&opp.shield_capsule()),
            front,
            rear,
        ];
        for (k, pt) in pts.iter().enumerate() {
            let r = rel(*pt);
            opp_in[k * 2] = r[0];
            opp_in[k * 2 + 1] = r[1];
        }
        // Sword block (5 values) at 16..21.
        let osw = opp.sword_capsule();
        let ospv = opp.sword_prev_capsule();
        let tv = [(osw.b[0] - ospv.b[0]) / PHYS_DT, (osw.b[1] - ospv.b[1]) / PHYS_DT];
        let rv = relv(tv);
        opp_in[16] = rv[0];
        opp_in[17] = rv[1];
        let to_me = [o[0] - osw.b[0], o[1] - osw.b[1]];
        let dist = (to_me[0] * to_me[0] + to_me[1] * to_me[1]).sqrt().max(1e-3);
        let closing = (tv[0] * to_me[0] + tv[1] * to_me[1]) / dist;
        opp_in[18] = closing / VEL_SCALE;
        opp_in[19] = if closing > 0.5 { (dist / closing).min(2.0) / 2.0 } else { 1.0 };
        opp_in[20] = (osw.b[1] - o[1]) / POS_SCALE;
        // Opponent hp (13) at 21..34.
        for pt in Part::ALL {
            opp_in[21 + pt as usize] = opp.hp_frac(pt);
        }
    }
    let hp_start = 21;
    for k in 0..34 {
        let mut v = opp_in[k];
        if blind {
            v = 0.0;
        } else if noise > 0.0 && k < hp_start {
            v += (rng.f32() * 2.0 - 1.0) * noise;
        }
        opp_in[k] = v;
    }
    for k in 0..21 {
        w.put(opp_in[k]);
    }
    debug_assert_eq!(w.i, OPP_HP.start);
    for k in 21..34 {
        w.put(opp_in[k]);
    }
    // Opponent posture, stagger flag, distance (3).
    w.put(if blind { 0.0 } else { opp.posture / 100.0 });
    w.put(if !blind && opp.state() == State::Staggered { 1.0 } else { 0.0 });
    w.put((opp.x - me.x).abs() / 14.0);
    // Walls (2) and line of sight (1).
    let (ahead, behind) = wall_distances(me, arena);
    w.put(ahead / 14.0);
    w.put(behind / 14.0);
    w.put(if line_of_sight_blocked(o[0], mid(&opp.capsule(Part::Torso))[0], arena) { 1.0 } else { 0.0 });
    debug_assert_eq!(w.i, BASE_IN);
    // Own move state (always known to the fighter).
    for k in 0..14 {
        w.put(if mine.id.index() == k { 1.0 } else { 0.0 });
    }
    for k in 0..4 {
        w.put(if mine.phase.index() == k { 1.0 } else { 0.0 });
    }
    w.put(mine.progress);
    w.put(if mine.committed { 1.0 } else { 0.0 });
    w.put(if mine.open { 1.0 } else { 0.0 });
    w.put(mine.riposte_remaining / 0.6);
    w.put(mine.guard_broken_remaining / 0.4);
    w.put(mine.last_cost / 30.0);
    // Opponent telegraph (zero when blind).
    for k in 0..14 {
        w.put(if !blind && theirs.id.index() == k { 1.0 } else { 0.0 });
    }
    for k in 0..4 {
        w.put(if !blind && theirs.phase.index() == k { 1.0 } else { 0.0 });
    }
    w.put(if blind { 0.0 } else { theirs.progress });
    w.put(if !blind && theirs.open { 1.0 } else { 0.0 });
    w.put(if blind { 0.0 } else { theirs.recovery_remaining / 0.5 });
    debug_assert_eq!(w.i, N_IN);
    w.x
}
