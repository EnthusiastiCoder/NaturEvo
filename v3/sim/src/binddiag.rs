//! Measurement of blade binds and body overlap in fights (diagnostics only, never read by fitness).
use crate::combat::swept_pair;
use crate::duel::{Brain, Match};
use crate::robot::{Part, Robot};

#[derive(Clone, Debug, Default)]
pub struct BindStats {
    pub fights: u32,
    pub ticks: f64,
    /// Ticks with the two blades touching.
    pub blade_touch: f64,
    /// Ticks with the blades touching and both blade tips slower than 1 m/s (a static bind).
    pub static_bind: f64,
    /// Longest run of consecutive static-bind ticks, summed over fights.
    pub longest_bind: f64,
    /// Ticks with the two bodies (head, torso, pelvis capsules) overlapping.
    pub body_overlap: f64,
    /// Number of times the fighters swapped sides.
    pub swaps: f64,
    pub swings: f64,
    pub hits: f64,
    /// Hits dealt while the attacker was not in an attack move's windup or active phase (blade drift, chip).
    pub drift_hits: f64,
}

pub fn bodies_overlap(a: &Robot, b: &Robot) -> bool {
    for p in [Part::Head, Part::Torso, Part::Pelvis] {
        for q in [Part::Head, Part::Torso, Part::Pelvis] {
            let (x, y) = (a.capsule(p), b.capsule(q));
            if swept_pair(x, x, y, y).is_some() {
                return true;
            }
        }
    }
    false
}

fn tip_speed(r: &Robot) -> f32 {
    let (s, p) = (r.sword_capsule(), r.sword_prev_capsule());
    let d = [s.b[0] - p.b[0], s.b[1] - p.b[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt() * 60.0
}

pub fn play(a: Brain, b: Brain, seed: u64, flat: bool, s: &mut BindStats) {
    let mut m = Match::new(seed, a, b);
    if flat {
        m.arena = crate::arena::Arena::flat(m.arena.width);
    }
    m.disable_grace_trace();
    let (mut run, mut best) = (0u32, 0u32);
    let mut seen_hits = 0usize;
    let mut side = (m.robots[0].x - m.robots[1].x).signum();
    while !m.is_over() {
        m.step();
        s.ticks += 1.0;
        while seen_hits < m.hit_log().len() {
            let (_, att) = m.hit_log()[seen_hits];
            seen_hits += 1;
            let mi = m.move_info(att as usize);
            if !(crate::skillbot::is_attack(mi.id) && matches!(mi.phase, crate::moveset::Phase::Windup | crate::moveset::Phase::Active)) {
                s.drift_hits += 1.0;
            }
        }
        let (r0, r1) = (&m.robots[0], &m.robots[1]);
        let (sa, sb) = (r0.sword_capsule(), r1.sword_capsule());
        let touch = swept_pair(sa, sa, sb, sb).is_some();
        if touch {
            s.blade_touch += 1.0;
        }
        if touch && tip_speed(r0) < 1.0 && tip_speed(r1) < 1.0 {
            s.static_bind += 1.0;
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
        if bodies_overlap(r0, r1) {
            s.body_overlap += 1.0;
        }
        let sd = (r0.x - r1.x).signum();
        if sd != side && sd != 0.0 {
            s.swaps += 1.0;
            side = sd;
        }
    }
    let r = m.result();
    s.fights += 1;
    s.longest_bind += best as f64;
    s.swings += (r.swings[0] + r.swings[1]) as f64;
    s.hits += (r.hits[0] + r.hits[1]) as f64;
}
