//! Combat: swept capsule collision, sword clash/deflect/perfect parry, shield block, body hits,
//! knockback, hit-stop, posture. The attacker sword is swept against the defender sword,
//! shield plate and body capsules; the earliest contact in the tick wins (ties: clash > block > body).
use crate::arena::Arena;
use crate::config::{IMPACT_CAP, PHYS_DT};
use crate::moves::{alignment, balance_factor, health_factor, raw_impact, Move};
use crate::robot::{Capsule, Part, Robot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClashKind {
    Deflect,
    Clash,
    PerfectParry,
}

impl ClashKind {
    pub fn log_name(&self) -> &'static str {
        match self {
            ClashKind::Deflect => "parry",
            ClashKind::Clash => "clash",
            ClashKind::PerfectParry => "perfect_parry",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HitEvent {
    pub part: Part,
    pub point: [f32; 2],
    /// Direction of the relative velocity at the contact point (unit).
    pub normal: [f32; 2],
    pub impact: f32,
    pub damage: f32,
    pub zone: f32,
    /// Knockback velocity applied to the defender root (signed, +x away from an attacker on the left).
    pub knockback: f32,
    pub hitstop_ticks: u32,
}

#[derive(Clone, Debug)]
pub enum CombatEvent {
    Hit(HitEvent),
    Clash { kind: ClashKind, point: [f32; 2], normal: [f32; 2] },
    Block { raw: f32, reduced: f32, point: [f32; 2], normal: [f32; 2] },
}

/// Result of a swept capsule test.
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub point: [f32; 2],
    /// Unit direction of the relative velocity at the contact point.
    pub normal: [f32; 2],
    /// Relative displacement at the contact point over the tick.
    pub velocity: [f32; 2],
    /// Blade parameter 0..1 of the contact.
    pub s: f32,
    /// Target axis parameter 0..1 of the contact.
    pub u: f32,
    /// Swept fraction of the tick at first contact.
    pub t: f32,
    /// Unit vector from the blade contact point toward the target axis point.
    pub n_hit: [f32; 2],
}

const SAMPLES: usize = 8;
const KNOCKBACK_K: f32 = 20.0;
const MIN_SPEED: f32 = 1.0;
const BLOCK_REDUCTION: f32 = 0.85;

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
fn len(a: [f32; 2]) -> f32 {
    dot(a, a).sqrt()
}
fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn lerp_cap(a: &Capsule, b: &Capsule, t: f32) -> Capsule {
    Capsule { a: lerp(a.a, b.a, t), b: lerp(a.b, b.b, t), r: a.r + (b.r - a.r) * t }
}
fn unit(a: [f32; 2]) -> Option<[f32; 2]> {
    let l = len(a);
    if l < 1e-6 {
        None
    } else {
        Some([a[0] / l, a[1] / l])
    }
}

/// Closest points between segments p1-q1 and p2-q2: (s, t, distance).
fn seg_seg(p1: [f32; 2], q1: [f32; 2], p2: [f32; 2], q2: [f32; 2]) -> (f32, f32, f32) {
    let d1 = sub(q1, p1);
    let d2 = sub(q2, p2);
    let r = sub(p1, p2);
    let a = dot(d1, d1);
    let e = dot(d2, d2);
    let f = dot(d2, r);
    let eps = 1e-9;
    let mut s;
    let mut t;
    if a <= eps && e <= eps {
        s = 0.0;
        t = 0.0;
    } else if a <= eps {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = dot(d1, r);
        if e <= eps {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = dot(d1, d2);
            let denom = a * e - b * b;
            s = if denom > 1e-6 * a * e { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.5 };
            t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
        }
    }
    let c1 = lerp(p1, q1, s);
    let c2 = lerp(p2, q2, t);
    (s, t, len(sub(c1, c2)))
}

/// Sweep a moving capsule against a moving target (both given at start and end of the tick).
pub fn swept_pair(prev: Capsule, cur: Capsule, tprev: Capsule, tcur: Capsule) -> Option<Contact> {
    for i in 1..=SAMPLES {
        let t = i as f32 / SAMPLES as f32;
        let b = lerp_cap(&prev, &cur, t);
        let g = lerp_cap(&tprev, &tcur, t);
        let (s, u, d) = seg_seg(b.a, b.b, g.a, g.b);
        if d <= b.r + g.r {
            let bv = sub(lerp(cur.a, cur.b, s), lerp(prev.a, prev.b, s));
            let tv = sub(lerp(tcur.a, tcur.b, u), lerp(tprev.a, tprev.b, u));
            let vel = sub(bv, tv);
            let point = lerp(b.a, b.b, s);
            let tpoint = lerp(g.a, g.b, u);
            let n_hit = unit(sub(tpoint, point)).or_else(|| unit(vel)).unwrap_or([1.0, 0.0]);
            let normal = unit(vel).unwrap_or(n_hit);
            return Some(Contact { point, normal, velocity: vel, s, u, t, n_hit });
        }
    }
    None
}

/// Sweep a moving capsule against a static target.
pub fn swept_hit(prev: Capsule, cur: Capsule, target: Capsule) -> Option<Contact> {
    swept_pair(prev, cur, target, target)
}

/// Hit-stop length for an impact: 3..=6 ticks, monotonic.
pub fn hitstop_ticks(impact: f32) -> u32 {
    3 + (3.0 * (impact / IMPACT_CAP).clamp(0.0, 1.0)).round() as u32
}

fn blade_dir(c: &Capsule) -> [f32; 2] {
    unit(sub(c.b, c.a)).unwrap_or([1.0, 0.0])
}

enum Target {
    Clash(Contact),
    Shield(Contact),
    Body(Part, Contact),
}

/// Resolve one attacker swing for this tick. Mutates both robots (hp, posture, shield, blade speeds,
/// knockback) and returns the events. Call twice with roles swapped for a full tick.
/// Not modelled here: kick and shield-bash sweeps (no previous-tick capsules for feet or shield).
pub fn resolve_tick(att: &mut Robot, def: &mut Robot, arena: &Arena) -> Vec<CombatEvent> {
    resolve_tick_with(att, def, arena, &Mods::default())
}

/// Rule modifiers for one resolved swing (see `rules.rs`): all neutral by default.
#[derive(Clone, Copy, Debug)]
pub struct Mods {
    /// Extra factor on the impact of a body hit (the defender's own exposed factor is applied by `damage_part`).
    pub impact_mult: f32,
    /// Factor on the posture a body hit adds.
    pub posture_mult: f32,
    /// Factor on the posture a shield hit adds to the defender.
    pub block_posture_mult: f32,
    /// The shield cannot be hit (guard broken): the swing passes to the body.
    pub shield_ignored: bool,
}

impl Default for Mods {
    fn default() -> Self {
        Mods { impact_mult: 1.0, posture_mult: 1.0, block_posture_mult: 1.0, shield_ignored: false }
    }
}

pub fn resolve_tick_with(att: &mut Robot, def: &mut Robot, arena: &Arena, mods: &Mods) -> Vec<CombatEvent> {
    let mut events = Vec::new();
    if !att.has_sword() || !att.can_act() || def.is_ko() {
        return events;
    }
    let sp = att.sword_prev_capsule();
    let sc = att.sword_capsule();

    let mut best: Option<(f32, u8, Target)> = None;
    let mut consider = |t: f32, pri: u8, tg: Target| {
        let better = match &best {
            None => true,
            Some((bt, bp, _)) => t < *bt - 1e-6 || ((t - *bt).abs() <= 1e-6 && pri < *bp),
        };
        if better {
            best = Some((t, pri, tg));
        }
    };

    if def.has_sword() {
        if let Some(c) = swept_pair(sp, sc, def.sword_prev_capsule(), def.sword_capsule()) {
            if len(sub(sc.a, sp.a)) / PHYS_DT >= MIN_SPEED || len(sub(sc.b, sp.b)) / PHYS_DT >= MIN_SPEED {
                consider(c.t, 0, Target::Clash(c));
            }
        }
    }
    if def.shield_active() && !mods.shield_ignored {
        if let Some(c) = swept_hit(sp, sc, def.shield_capsule()) {
            if len(c.velocity) / PHYS_DT >= MIN_SPEED {
                consider(c.t, 1, Target::Shield(c));
            }
        }
    }
    // Core parts (head, torso, pelvis) compete with clash and block by time; limbs only count when
    // nothing else is struck this tick (a swing at the body is not stopped by the dangling arms).
    let mut limb: Option<(f32, Part, Contact)> = None;
    for p in Part::ALL {
        if def.is_detached(p) {
            continue;
        }
        let mut hit = swept_hit(sp, sc, def.capsule(p));
        if p == Part::Head && !def.is_detached(Part::Torso) {
            // The neck is part of the head hit zone.
            if let Some(n) = swept_hit(sp, sc, def.neck_capsule()) {
                if hit.as_ref().map_or(true, |h| n.t < h.t) {
                    hit = Some(n);
                }
            }
        }
        if let Some(c) = hit {
            if len(c.velocity) / PHYS_DT >= MIN_SPEED {
                if matches!(p, Part::Head | Part::Torso | Part::Pelvis) {
                    consider(c.t, if p == Part::Torso { 2 } else { 3 }, Target::Body(p, c));
                } else if limb.as_ref().map_or(true, |(t, _, _)| c.t < *t) {
                    limb = Some((c.t, p, c));
                }
            }
        }
    }

    let target = match best {
        Some((_, _, t)) => t,
        None => match limb {
            Some((_, p, c)) => Target::Body(p, c),
            None => return events,
        },
    };
    match target {
        Target::Clash(c) => {
            let dc = def.sword_capsule();
            let dp = def.sword_prev_capsule();
            let perp_ok = dot(blade_dir(&sc), blade_dir(&dc)).abs() <= (35.0f32).to_radians().sin();
            let dvel = sub(lerp(dc.a, dc.b, c.u), lerp(dp.a, dp.b, c.u));
            let avel = sub(lerp(sc.a, sc.b, c.s), lerp(sp.a, sp.b, c.s));
            let toward = match unit(avel) {
                Some(ah) => -dot(dvel, ah) / PHYS_DT,
                None => 0.0,
            };
            let mut kind = ClashKind::Clash;
            if perp_ok && toward > 1.0 {
                kind = if def.sword_moving_since > 0.0 && def.sword_moving_since < 0.25 {
                    ClashKind::PerfectParry
                } else {
                    ClashKind::Deflect
                };
            }
            match kind {
                ClashKind::Clash => {
                    att.sword_ang_vel *= 0.5;
                    def.sword_ang_vel *= 0.5;
                    att.add_posture(15.0);
                    def.add_posture(15.0);
                }
                ClashKind::Deflect | ClashKind::PerfectParry => {
                    att.sword_ang_vel *= -0.4;
                    att.add_posture(25.0 * att.strike_quality);
                    def.add_posture(5.0);
                    if kind == ClashKind::PerfectParry {
                        att.add_posture(40.0);
                        def.free_regen_time = 0.3;
                    }
                }
            }
            events.push(CombatEvent::Clash { kind, point: c.point, normal: c.normal });
        }
        Target::Shield(c) => {
            let raw = strike_impact(att, &c);
            let reduced = raw * (1.0 - BLOCK_REDUCTION);
            let frac = (def.shield_integrity / 100.0).clamp(0.0, 1.0);
            def.shield_integrity = (def.shield_integrity - 0.3 * raw).max(0.0);
            def.damage_part(Part::FArmL, reduced);
            att.add_posture(10.0);
            def.add_posture(10.0 * (1.0 - frac * 0.5) * mods.block_posture_mult);
            events.push(CombatEvent::Block { raw, reduced, point: c.point, normal: c.normal });
        }
        Target::Body(part, c) => {
            let impact = strike_impact(att, &c);
            let zone = def.zone_mult(part);
            let damage = def.damage_part(part, impact);
            def.add_posture(impact * 0.05);
            let knockback = KNOCKBACK_K * impact * c.normal[0] / def.total_mass();
            def.x = arena.clamp_x(def.x + knockback * 0.1);
            def.apply_impulse(knockback * 0.3);
            events.push(CombatEvent::Hit(HitEvent {
                part,
                point: c.point,
                normal: c.normal,
                impact,
                damage,
                zone,
                knockback,
                hitstop_ticks: hitstop_ticks(impact),
            }));
        }
    }
    events
}

/// Raw impact of the attacker sword at a contact (m_eff = sword plus a share of the arm).
fn strike_impact(att: &Robot, c: &Contact) -> f32 {
    let v = len(c.velocity) / PHYS_DT;
    let a = alignment(c.velocity, c.n_hit);
    let m_eff = 25.0 + 10.0;
    let tm = att.strike_quality.clamp(0.6, 1.0);
    let h = health_factor(att, Move::Slash);
    let b = balance_factor(att.balance_margin(), att.posture, false);
    raw_impact(m_eff, v, a, tm, h, b)
}
