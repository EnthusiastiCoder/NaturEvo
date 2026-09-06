//! Sensors: 40 normalised inputs per fighter, all in [-1, 1].
//! 0 hp, 1 stamina, 2 dodge cooldown, 3 own timer, 4-10 own state one-hot,
//! 11 enemy distance, 12/13 sin/cos bearing to enemy, 14 enemy hp, 15 enemy stamina,
//! 16-22 enemy state one-hot, 23 enemy timer, 24/25 enemy facing relative sin/cos,
//! 26/27 own position, 28-39 wall rays across the field of view.
use crate::fighter::{CombatState, Role};
use crate::matchup::{Match, DODGE_COOLDOWN};

pub const N_IN: usize = 40;
const MAX_RAYS: usize = 9; // slots 28-36; n_rays (default 9) fills them, rest stay 0
const STEP: f32 = 0.5;

fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI
}

fn occluded(a: &crate::world::Arena, x0: f32, y0: f32, x1: f32, y1: f32, d: f32) -> bool {
    let n = (d / STEP) as usize;
    (1..n).any(|i| {
        let t = i as f32 * STEP / d;
        a.is_blocked(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t)
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
    }
}

fn unit(v: f32) -> f32 {
    if v.is_finite() { v.clamp(-1.0, 1.0) } else { 0.0 }
}

pub fn observe(m: &Match, role: Role) -> [f32; N_IN] {
    let cfg = m.cfg();
    let me = m.fighter(role);
    let en = m.fighter(role.other());
    let mut o = [0.0f32; N_IN];
    o[0] = me.hp / cfg.max_hp;
    o[1] = me.stamina / cfg.max_stamina;
    o[2] = me.dodge_cooldown as f32 / DODGE_COOLDOWN as f32;
    o[3] = me.timer as f32 / 10.0;
    o[4 + state_idx(me.state)] = 1.0;
    let (dx, dy) = (en.x - me.x, en.y - me.y);
    let dist = (dx * dx + dy * dy).sqrt();
    let bearing = wrap(dy.atan2(dx) - me.facing);
    let arena = m.arena();
    let seen = dist <= cfg.vision_range
        && bearing.abs() <= cfg.vision_fov / 2.0
        && !occluded(arena, me.x, me.y, en.x, en.y, dist);
    if seen {
        o[11] = dist / cfg.vision_range;
        o[12] = bearing.sin();
        o[13] = bearing.cos();
        o[14] = en.hp / cfg.max_hp;
        o[15] = en.stamina / cfg.max_stamina;
        o[16 + state_idx(en.state)] = 1.0;
        o[23] = en.timer as f32 / 10.0;
        let rel = en.facing - me.facing;
        o[24] = rel.sin();
        o[25] = rel.cos();
        o[37] = 1.0;
        o[38] = if dist <= crate::matchup::ATTACK_RANGE { 1.0 } else { 0.0 };
    }
    o[39] = 1.0; // bias
    o[26] = me.x / cfg.arena_w * 2.0 - 1.0;
    o[27] = me.y / cfg.arena_h * 2.0 - 1.0;
    let n_rays = cfg.n_rays.min(MAX_RAYS).max(1);
    for k in 0..n_rays {
        let t = if n_rays > 1 { k as f32 / (n_rays - 1) as f32 - 0.5 } else { 0.0 };
        let a = me.facing + t * cfg.vision_fov;
        let (s, c) = a.sin_cos();
        let mut d = cfg.vision_range;
        let mut r = STEP;
        while r < cfg.vision_range {
            if arena.is_blocked(me.x + c * r, me.y + s * r) {
                d = r;
                break;
            }
            r += STEP;
        }
        o[28 + k] = d / cfg.vision_range;
    }
    for v in o.iter_mut() {
        *v = unit(*v);
    }
    o
}
