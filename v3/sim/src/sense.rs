//! Sensing: the network input vector, in the fighter's own facing frame (forward = +x).
//! The opponent is always visible (no FOV). A damaged head adds deterministic noise to opponent
//! inputs; a disabled head blinds the fighter (opponent inputs are zero).
use crate::arena::Arena;
use crate::config::PHYS_DT;
use crate::rng::Rng;
use crate::robot::{Capsule, Joint, Part, Robot, State};
use std::ops::Range;

/// Number of normalised network inputs.
pub const N_IN: usize = 104;

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
