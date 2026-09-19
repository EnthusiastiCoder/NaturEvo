//! Sensors: 80 normalised inputs per fighter, all in [-1, 1]. Layout in `idx`.
//! The enemy block (ENEMY_DIST..=IN_RANGE) is zero unless the enemy is seen.
use crate::fighter::{CombatState, Fighter, Role};
use crate::matchup::Match;
use crate::status::Status;

pub const N_IN: usize = 80;

pub mod idx {
    pub const HP: usize = 0;
    pub const STAMINA: usize = 1;
    pub const DODGE_CD: usize = 2;
    pub const OWN_PHASE: usize = 3;
    pub const OWN_STATE: usize = 4; // 12
    pub const OWN_FX: usize = 16; // 7
    pub const OWN_ATK: usize = 23; // 3
    pub const OWN_SPECIES: usize = 26; // 5
    pub const ENEMY_DIST: usize = 31;
    pub const ENEMY_SIN: usize = 32;
    pub const ENEMY_COS: usize = 33;
    pub const ENEMY_HP: usize = 34;
    pub const ENEMY_STAMINA: usize = 35;
    pub const ENEMY_STATE: usize = 36; // 12
    pub const ENEMY_PHASE: usize = 48;
    pub const ENEMY_FACE_SIN: usize = 49;
    pub const ENEMY_FACE_COS: usize = 50;
    pub const ENEMY_FX: usize = 51; // 7
    pub const ENEMY_ATK: usize = 58; // 3
    pub const ENEMY_SPECIES: usize = 61; // 5
    pub const SEEN: usize = 66;
    pub const IN_RANGE: usize = 67;
    pub const BIAS: usize = 68;
    pub const OWN_X: usize = 69;
    pub const OWN_Y: usize = 70;
    pub const RAYS: usize = 71; // 9
}

const MAX_RAYS: usize = 9; // slots 71-79; n_rays (default 9) fills them, rest stay 0
const STEP: f32 = 1.0; // ray_dist samples at unit steps; keep in sync

/// Normaliser for each status remaining-tick input (knock, stagger, stun, confused, slowed, bleed, poison).
const FX_MAX: [f32; 7] = [6.0, 20.0, 45.0, 90.0, 90.0, 150.0, 300.0];

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
    s as usize
}

/// Phase progress in (0, 1]: ticks left in the current phase over the phase length; 0 when no timed phase.
fn phase(m: &Match, role: Role, f: &Fighter) -> f32 {
    if f.timer == 0 {
        return 0.0;
    }
    let c = &m.cfg().combat;
    let ms = m.moveset(role);
    let mv = &ms[f.move_idx.min(ms.len() - 1)];
    let total = match f.state {
        CombatState::Windup => mv.windup,
        CombatState::Active => mv.active,
        CombatState::Recovery => (mv.recovery as f32 * c.whiff_recovery_mult).ceil() as u32,
        CombatState::Dodging => c.dodge_ticks,
        CombatState::GuardRaise => c.guard_raise_ticks,
        CombatState::GuardRecover => c.guard_break_stun.max(c.guard_recover_ticks),
        CombatState::Staggered => FX_MAX[Status::Stagger.idx()] as u32,
        CombatState::Stunned => FX_MAX[Status::Stun.idx()] as u32,
        CombatState::Knocked => FX_MAX[Status::Knock.idx()] as u32,
        _ => return 0.0,
    };
    f.timer as f32 / total.max(1) as f32
}

fn atk_kind(f: &Fighter) -> Option<usize> {
    matches!(f.state, CombatState::Windup | CombatState::Active | CombatState::Recovery).then_some(f.move_idx.min(2))
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
    use idx::*;
    let cfg = m.cfg();
    let me = m.fighter(role);
    let en = m.fighter(role.other());
    let mut o = [0.0f32; N_IN];
    o[HP] = me.hp / cfg.max_hp;
    o[STAMINA] = me.stamina / cfg.max_stamina;
    o[DODGE_CD] = me.dodge_cooldown as f32 / cfg.combat.dodge_cooldown as f32;
    o[OWN_PHASE] = phase(m, role, me);
    o[OWN_STATE + state_idx(me.state)] = 1.0;
    for k in 0..7 {
        o[OWN_FX + k] = me.status.remaining[k] as f32 / FX_MAX[k];
    }
    if let Some(k) = atk_kind(me) {
        o[OWN_ATK + k] = 1.0;
    }
    o[OWN_SPECIES + m.species(role).idx()] = 1.0;
    let (dx, dy) = (en.x - me.x, en.y - me.y);
    let dist = (dx * dx + dy * dy).sqrt();
    let (sf, cf) = me.facing.sin_cos();
    // Bearing to the enemy relative to facing, as (sin, cos) without atan2.
    let (bsin, bcos) = if dist > 1e-6 { ((cf * dy - sf * dx) / dist, (cf * dx + sf * dy) / dist) } else { (0.0, 1.0) };
    let (shf, chf, sd, cd) = fov_consts(cfg.vision_fov, cfg.n_rays.min(MAX_RAYS).max(1));
    let arena = m.arena();
    let seen = dist <= cfg.vision_range && bcos >= chf && !occluded(arena, me.x, me.y, en.x, en.y, dist);
    if seen {
        o[ENEMY_DIST] = dist / cfg.vision_range;
        o[ENEMY_SIN] = bsin;
        o[ENEMY_COS] = bcos;
        o[ENEMY_HP] = en.hp / cfg.max_hp;
        o[ENEMY_STAMINA] = en.stamina / cfg.max_stamina;
        o[ENEMY_STATE + state_idx(en.state)] = 1.0;
        o[ENEMY_PHASE] = phase(m, role.other(), en);
        let (se, ce) = en.facing.sin_cos();
        o[ENEMY_FACE_SIN] = se * cf - ce * sf; // sin(en.facing - me.facing)
        o[ENEMY_FACE_COS] = ce * cf + se * sf;
        for k in 0..7 {
            o[ENEMY_FX + k] = en.status.remaining[k] as f32 / FX_MAX[k];
        }
        if let Some(k) = atk_kind(en) {
            o[ENEMY_ATK + k] = 1.0;
        }
        o[ENEMY_SPECIES + m.species(role.other()).idx()] = 1.0;
        o[SEEN] = 1.0;
        let reach = m.moveset(role).iter().fold(0.0f32, |a, mv| a.max(mv.range));
        o[IN_RANGE] = if dist <= reach { 1.0 } else { 0.0 };
    }
    o[BIAS] = 1.0;
    o[OWN_X] = me.x / cfg.arena_w * 2.0 - 1.0;
    o[OWN_Y] = me.y / cfg.arena_h * 2.0 - 1.0;
    let n_rays = cfg.n_rays.min(MAX_RAYS).max(1);
    // Rays fan out evenly; rotate a unit vector instead of calling sin_cos per ray.
    // First ray sits half a field of view to the right of facing (angle facing - fov/2).
    let (mut s, mut c) = if n_rays > 1 { (sf * chf - cf * shf, cf * chf + sf * shf) } else { (sf, cf) };
    for k in 0..n_rays {
        let d = arena.ray_dist(me.x, me.y, c, s, cfg.vision_range);
        o[RAYS + k] = d / cfg.vision_range;
        let (ns, nc) = (s * cd + c * sd, c * cd - s * sd);
        s = ns;
        c = nc;
    }
    for v in o.iter_mut() {
        *v = unit(*v);
    }
    o
}
