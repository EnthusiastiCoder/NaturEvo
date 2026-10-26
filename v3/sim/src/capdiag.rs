//! Measurement of what trained nets do in capped fights (diagnostics only, never read by fitness).
use crate::duel::{Brain, Match};
use crate::genome::Genome;
use crate::moveset::MoveId;
use crate::skillbot::is_attack;
use std::path::Path;

#[derive(Default, Clone, Debug)]
pub struct CapStats {
    pub fights: u32,
    pub capped: u32,
    /// Sums over capped fights (per fighter 0 and 1).
    pub gap_sum: f64,
    pub gap_n: f64,
    pub swings: [f64; 2],
    pub parries: [f64; 2],
    pub guards: [f64; 2],
    pub idles: [f64; 2],
    pub ticks: f64,
    /// Fights where fighter 0 / 1 made the first attack swing.
    pub first: [u32; 2],
    pub no_attack: u32,
}

/// Play one fight and fold it into `s` if capped (or into `all` for every fight).
pub fn play(a: Brain, b: Brain, seed: u64, s: &mut CapStats, all: &mut CapStats) {
    let mut m = Match::new(seed, a, b);
    m.disable_grace_trace();
    let mut gap = 0.0f64;
    let mut n = 0.0f64;
    let mut seen = [0usize; 2];
    let mut first: Option<usize> = None;
    while !m.is_over() {
        m.step();
        gap += (m.robots[0].x - m.robots[1].x).abs() as f64;
        n += 1.0;
        if first.is_none() {
            for i in 0..2 {
                let st = m.move_starts(i);
                if st.len() > seen[i] {
                    if st[seen[i]..].iter().any(|&id| is_attack(id)) && first.is_none() {
                        first = Some(i);
                    }
                    seen[i] = st.len();
                }
            }
        }
    }
    let r = m.result();
    let fold = |t: &mut CapStats| {
        t.fights += 1;
        t.capped += r.capped as u32;
        t.gap_sum += gap;
        t.gap_n += n;
        t.ticks += n;
        for i in 0..2 {
            let st = m.move_starts(i);
            t.swings[i] += st.iter().filter(|&&id| is_attack(id)).count() as f64;
            t.parries[i] += st.iter().filter(|&&id| id == MoveId::Parry).count() as f64;
            t.guards[i] += st.iter().filter(|&&id| id == MoveId::Guard).count() as f64;
            t.idles[i] += st.iter().filter(|&&id| id == MoveId::Idle).count() as f64;
        }
        match first {
            Some(i) => t.first[i] += 1,
            None => t.no_attack += 1,
        }
    };
    fold(all);
    if r.capped {
        fold(s);
    }
}
