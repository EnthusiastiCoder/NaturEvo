//! Sensors: 40 normalised inputs per fighter, all in [-1, 1].
//! 0 hp, 1 stamina, 2 dodge cooldown, 3 own timer, 4-10 own state one-hot,
//! 11 enemy distance, 12/13 sin/cos bearing to enemy, 14 enemy hp, 15 enemy stamina,
//! 16-22 enemy state one-hot, 23 enemy timer, 24/25 enemy facing relative sin/cos,
//! 26/27 own position, 28-39 wall rays across the field of view.
use crate::fighter::{CombatState, Role};
use crate::matchup::{Match, DODGE_COOLDOWN};

pub const N_IN: usize = 40;
const N_RAYS: usize = 12;

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
    o[11] = dist / cfg.vision_range;
    let bearing = dy.atan2(dx) - me.facing;
    o[12] = bearing.sin();
    o[13] = bearing.cos();
    o[14] = en.hp / cfg.max_hp;
    o[15] = en.stamina / cfg.max_stamina;
    o[16 + state_idx(en.state)] = 1.0;
    o[23] = en.timer as f32 / 10.0;
    let rel = en.facing - me.facing;
    o[24] = rel.sin();
    o[25] = rel.cos();
    o[26] = me.x / cfg.arena_w * 2.0 - 1.0;
    o[27] = me.y / cfg.arena_h * 2.0 - 1.0;
    let arena = m.arena();
    let step = 0.25f32;
    for k in 0..N_RAYS {
        let t = k as f32 / (N_RAYS - 1) as f32 - 0.5;
        let a = me.facing + t * cfg.vision_fov;
        let (s, c) = a.sin_cos();
        let mut d = cfg.vision_range;
        let mut r = step;
        while r < cfg.vision_range {
            if arena.is_blocked(me.x + c * r, me.y + s * r) {
                d = r;
                break;
            }
            r += step;
        }
        o[28 + k] = d / cfg.vision_range;
    }
    for v in o.iter_mut() {
        *v = unit(*v);
    }
    o
}
