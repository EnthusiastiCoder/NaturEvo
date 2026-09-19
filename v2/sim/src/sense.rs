//! Sensors: 40 normalised inputs per fighter, all in [-1, 1].
//! 0 hp, 1 stamina, 2 dodge cooldown, 3 own timer, 4-10 own state one-hot,
//! 11 enemy distance, 12/13 sin/cos bearing to enemy, 14 enemy hp, 15 enemy stamina,
//! 16-22 enemy state one-hot, 23 enemy timer, 24/25 enemy facing relative sin/cos,
//! 26/27 own position, 28-39 wall rays across the field of view.
use crate::fighter::{CombatState, Role};
use crate::matchup::Match;

pub const N_IN: usize = 40;
const MAX_RAYS: usize = 9; // slots 28-36; n_rays (default 9) fills them, rest stay 0
const STEP: f32 = 1.0; // ray_dist samples at unit steps; keep in sync

fn occluded(a: &crate::world::Arena, x0: f32, y0: f32, x1: f32, y1: f32, d: f32) -> bool {
    if d <= STEP {
        return false;
    }
    // Samples at i*STEP for i in 1..n, n = (d/STEP) as usize.
    let n = (d / STEP) as usize;
    let (ux, uy) = ((x1 - x0) / d, (y1 - y0) / d);
    n > 1 && a.ray_dist(x0, y0, ux, uy, (n as f32 - 0.5) * STEP) < (n as f32 - 0.5) * STEP
}

/// (sin, cos) of half the field of view and of the angle between adjacent rays, memoised
/// per thread because the config is constant across a match.
fn fov_consts(fov: f32, n_rays: usize) -> (f32, f32, f32, f32) {
    thread_local! {
        static CACHE: std::cell::Cell<(f32, usize, (f32, f32, f32, f32))> =
            const { std::cell::Cell::new((f32::NAN, 0, (0.0, 0.0, 0.0, 0.0))) };
    }
    CACHE.with(|c| {
        let (f, n, v) = c.get();
        if f == fov && n == n_rays {
            return v;
        }
        let (sh, ch) = (0.5 * fov).sin_cos();
        let dstep = if n_rays > 1 { fov / (n_rays - 1) as f32 } else { 0.0 };
        let (sd, cd) = dstep.sin_cos();
        let v = (sh, ch, sd, cd);
        c.set((fov, n_rays, v));
        v
    })
}

fn state_idx(s: CombatState) -> usize {
    match s {
        CombatState::Idle => 0,
        CombatState::Windup => 1,
        CombatState::Active => 2,
        CombatState::Recovery => 3,
        CombatState::Blocking => 4,
        CombatState::Dodging => 5,
        CombatState::Dead => 6,
        // Temporary mapping until the sensor layout is rewritten for the new states.
        CombatState::GuardRaise | CombatState::GuardRecover => 4,
        CombatState::Staggered | CombatState::Stunned | CombatState::Knocked => 3,
    }
}

fn unit(v: f32) -> f32 {
    if v.is_finite() { v.clamp(-1.0, 1.0) } else { 0.0 }
}

/// Whether `role` currently sees the other fighter (range, field of view and line of sight).
pub fn sees(m: &Match, role: Role) -> bool {
    let cfg = m.cfg();
    let me = m.fighter(role);
    let en = m.fighter(role.other());
    let (dx, dy) = (en.x - me.x, en.y - me.y);
    let dist = (dx * dx + dy * dy).sqrt();
    let (sf, cf) = me.facing.sin_cos();
    let bcos = if dist > 1e-6 { (cf * dx + sf * dy) / dist } else { 1.0 };
    let (_, chf, _, _) = fov_consts(cfg.vision_fov, cfg.n_rays.min(MAX_RAYS).max(1));
    dist <= cfg.vision_range && bcos >= chf && !occluded(m.arena(), me.x, me.y, en.x, en.y, dist)
}

pub fn observe(m: &Match, role: Role) -> [f32; N_IN] {
    let cfg = m.cfg();
    let me = m.fighter(role);
    let en = m.fighter(role.other());
    let mut o = [0.0f32; N_IN];
    o[0] = me.hp / cfg.max_hp;
    o[1] = me.stamina / cfg.max_stamina;
    o[2] = me.dodge_cooldown as f32 / cfg.combat.dodge_cooldown as f32;
    o[3] = me.timer as f32 / 10.0;
    o[4 + state_idx(me.state)] = 1.0;
    let (dx, dy) = (en.x - me.x, en.y - me.y);
    let dist = (dx * dx + dy * dy).sqrt();
    let (sf, cf) = me.facing.sin_cos();
    // Bearing to the enemy relative to facing, as (sin, cos) without atan2.
    let (bsin, bcos) = if dist > 1e-6 { ((cf * dy - sf * dx) / dist, (cf * dx + sf * dy) / dist) } else { (0.0, 1.0) };
    let (shf, chf, sd, cd) = fov_consts(cfg.vision_fov, cfg.n_rays.min(MAX_RAYS).max(1));
    let arena = m.arena();
    let seen = dist <= cfg.vision_range && bcos >= chf && !occluded(arena, me.x, me.y, en.x, en.y, dist);
    if seen {
        o[11] = dist / cfg.vision_range;
        o[12] = bsin;
        o[13] = bcos;
        o[14] = en.hp / cfg.max_hp;
        o[15] = en.stamina / cfg.max_stamina;
        o[16 + state_idx(en.state)] = 1.0;
        o[23] = en.timer as f32 / 10.0;
        let (se, ce) = en.facing.sin_cos();
        o[24] = se * cf - ce * sf; // sin(en.facing - me.facing)
        o[25] = ce * cf + se * sf;
        o[37] = 1.0;
        o[38] = if dist <= 1.2 { 1.0 } else { 0.0 };
    }
    o[39] = 1.0; // bias
    o[26] = me.x / cfg.arena_w * 2.0 - 1.0;
    o[27] = me.y / cfg.arena_h * 2.0 - 1.0;
    let n_rays = cfg.n_rays.min(MAX_RAYS).max(1);
    // Rays fan out evenly; rotate a unit vector instead of calling sin_cos per ray.
    // First ray sits half a field of view to the right of facing (angle facing - fov/2).
    let (mut s, mut c) = if n_rays > 1 { (sf * chf - cf * shf, cf * chf + sf * shf) } else { (sf, cf) };
    for k in 0..n_rays {
        let d = arena.ray_dist(me.x, me.y, c, s, cfg.vision_range);
        o[28 + k] = d / cfg.vision_range;
        let (ns, nc) = (s * cd + c * sd, c * cd - s * sd);
        s = ns;
        c = nc;
    }
    for v in o.iter_mut() {
        *v = unit(*v);
    }
    o
}
