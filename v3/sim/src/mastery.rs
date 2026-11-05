//! Mastery measurement: per pairing over fixed seeds, how a fight looks (length, activity, binds, stagger loops, idling).
//! Diagnostics only; nothing here feeds back into fitness.
use crate::combat::swept_pair;
use crate::duel::{Brain, Match};
use crate::moveset::Phase;
use crate::robot::{Robot, State};
use crate::skillbot::is_attack;

#[derive(Clone, Debug, Default)]
pub struct Mastery {
    pub lens: Vec<u32>,
    pub capped: u32,
    pub swings: f64,
    pub hits: f64,
    pub distinct: f64,
    pub defences: f64,
    pub parries: f64,
    pub ticks: f64,
    pub both_stag: f64,
    pub bind: f64,
    pub hits_while_stag: f64,
    /// Ticks with both fighters in an Idle move at torso distance over 1.2 m (nobody doing anything).
    pub both_idle_far: f64,
    /// Longest run of consecutive both-idle ticks, summed over fights.
    pub longest_idle: f64,
    /// Blade-blade: ticks the blade capsules overlap at the end of the tick, ticks where the blade centre lines truly cross,
    /// of those, ticks with no clash event that tick, and the deepest penetration (m).
    pub blade_overlap: f64,
    pub blade_cross: f64,
    pub blade_noclash: f64,
    pub blade_cross_noclash: f64,
    pub max_depth: f64,
    pub wins0: u32,
    pub draws: u32,
}

fn tip_speed(r: &Robot) -> f32 {
    let (s, p) = (r.sword_capsule(), r.sword_prev_capsule());
    let d = [s.b[0] - p.b[0], s.b[1] - p.b[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt() * 60.0
}

pub fn play(a: Brain, b: Brain, seed: u64, flat: bool, s: &mut Mastery) {
    let mut m = Match::new(seed, a, b);
    if flat {
        m.arena = crate::arena::Arena::flat(m.arena.width);
    }
    let mut seen = 0usize;
    let (mut run, mut best) = (0u32, 0u32);
    while !m.is_over() {
        let clashes_before = m.clash_count;
        let stag_before = [m.robots[0].state() == State::Staggered, m.robots[1].state() == State::Staggered];
        m.step();
        s.ticks += 1.0;
        while seen < m.hit_log().len() {
            let (_, att) = m.hit_log()[seen];
            seen += 1;
            if stag_before[1 - att as usize] {
                s.hits_while_stag += 1.0;
            }
        }
        let (r0, r1) = (&m.robots[0], &m.robots[1]);
        if stag_before[0] && stag_before[1] {
            s.both_stag += 1.0;
        }
        let (sa, sb) = (r0.sword_capsule(), r1.sword_capsule());
        if swept_pair(sa, sa, sb, sb).is_some() && tip_speed(r0) < 1.0 && tip_speed(r1) < 1.0 {
            s.bind += 1.0;
        }
        {
            let (ca, cb) = (r0.sword_capsule(), r1.sword_capsule());
            let (_, _, d) = crate::combat::seg_seg(ca.a, ca.b, cb.a, cb.b);
            let rs = ca.r + cb.r;
            let clashed = m.clash_count != clashes_before;
            let in_play = !matches!(r0.state(), State::Fallen | State::Ko) && !matches!(r1.state(), State::Fallen | State::Ko);
            if d <= rs && in_play {
                s.blade_overlap += 1.0;
                if !clashed {
                    s.blade_noclash += 1.0;
                }
                // Penetration: how far one blade reaches past the other's centre line (distance to the nearer blade end
                // from the closest point), or the capsule overlap when the centre lines do not cross.
                let (si, ti, _) = crate::combat::seg_seg(ca.a, ca.b, cb.a, cb.b);
                let la = ((ca.b[0] - ca.a[0]).powi(2) + (ca.b[1] - ca.a[1]).powi(2)).sqrt();
                let lb = ((cb.b[0] - cb.a[0]).powi(2) + (cb.b[1] - cb.a[1]).powi(2)).sqrt();
                let depth = if d < 1e-4 {
                    s.blade_cross += 1.0;
                    if !clashed {
                        s.blade_cross_noclash += 1.0;
                    }
                    (rs + (si * la).min((1.0 - si) * la).min(ti * lb).min((1.0 - ti) * lb)) as f64
                } else {
                    (rs - d) as f64
                };
                s.max_depth = s.max_depth.max(depth);
            }
