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
        }
        let (i0, i1) = (m.move_info(0), m.move_info(1));
        let far = (r0.x - r1.x).abs() > 1.2;
        if far && i0.phase == Phase::Idle && i1.phase == Phase::Idle && r0.state() == State::Idle && r1.state() == State::Idle {
            s.both_idle_far += 1.0;
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
        let _ = is_attack;
    }
    let r = m.result();
    s.lens.push(r.ticks);
    s.capped += r.capped as u32;
    for who in 0..2 {
        s.swings += r.swings[who] as f64;
        s.hits += r.hits[who] as f64;
        let g = crate::grace::match_grace(&m, who);
        s.distinct += g.usage.iter().filter(|u| **u > 0.0).count() as f64;
        let d = &r.defence[who];
        s.defences += (d.parry + d.block + d.dodge) as f64;
        s.parries += d.parry as f64;
    }
    s.longest_idle += best as f64;
    match r.outcome {
        crate::duel::Outcome::WinA => s.wins0 += 1,
        crate::duel::Outcome::Draw => s.draws += 1,
        _ => {}
    }
}

impl Mastery {
    pub fn merge(&mut self, o: &Mastery) {
        self.lens.extend_from_slice(&o.lens);
        self.capped += o.capped;
        self.max_depth = self.max_depth.max(o.max_depth);
        self.wins0 += o.wins0;
        self.draws += o.draws;
        for (a, b) in [
            (&mut self.swings, o.swings), (&mut self.hits, o.hits), (&mut self.distinct, o.distinct), (&mut self.defences, o.defences),
            (&mut self.parries, o.parries), (&mut self.ticks, o.ticks), (&mut self.both_stag, o.both_stag), (&mut self.bind, o.bind),
            (&mut self.hits_while_stag, o.hits_while_stag), (&mut self.blade_overlap, o.blade_overlap), (&mut self.blade_cross, o.blade_cross), (&mut self.blade_noclash, o.blade_noclash), (&mut self.blade_cross_noclash, o.blade_cross_noclash), (&mut self.both_idle_far, o.both_idle_far), (&mut self.longest_idle, o.longest_idle),
        ] {
            *a += b;
        }
    }
    pub fn median(&self) -> f64 {
        let mut v = self.lens.clone();
        v.sort();
        if v.is_empty() { 0.0 } else { v[v.len() / 2] as f64 }
    }
    pub fn row(&self, label: &str) -> String {
        let n = self.lens.len().max(1) as f64;
        let t = self.ticks.max(1.0);
        let mean = self.lens.iter().map(|x| *x as f64).sum::<f64>() / n;
        format!(
            "| {} | {} | {:.0} | {:.0} | {:.1}% | {:.1} | {:.2} | {:.1} | {:.0}% | {:.1}% | {:.1}% | {:.1}% | {:.2} | {:.0} | {:.2} | {:.2} | {:.2} | {:.2} | {:.3} |\n",
            label, self.lens.len(), self.median(), mean, self.capped as f64 / n * 100.0,
            self.swings / n / 2.0, self.hits / n, self.distinct / n / 2.0,
            if self.defences > 0.0 { self.parries / self.defences * 100.0 } else { 0.0 },
            self.both_stag / t * 100.0, self.bind / t * 100.0, self.both_idle_far / t * 100.0, self.hits_while_stag / n, self.longest_idle / n,
            self.blade_overlap / n, self.blade_cross / n, self.blade_noclash / n, self.blade_cross_noclash / n, self.max_depth
        )
    }
}

pub const HEAD: &str = "| pair | fights | median ticks | mean ticks | capped | swings/fighter | hits/fight | distinct moves/fighter | parry share | both-staggered | bind | both idle far | hits taken while staggered/fight | longest idle run |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";

/// All ordered pairs of `names` (brains given), `seeds` fights each, in parallel over pairs.
pub fn table(cs: &[(String, Brain)], seeds: u64, flat: bool) -> String {
    let jobs: Vec<(usize, usize)> = (0..cs.len()).flat_map(|i| (0..cs.len()).map(move |j| (i, j))).collect();
    let res: Vec<Mastery> = std::thread::scope(|sc| {
        let hs: Vec<_> = jobs
            .iter()
            .map(|&(i, j)| {
                let (a, b) = (cs[i].1.clone(), cs[j].1.clone());
                sc.spawn(move || {
                    let mut s = Mastery::default();
                    for sd in 0..seeds {
                        play(a.clone(), b.clone(), 1000 + sd, flat, &mut s);
                    }
                    s
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut out = String::from(HEAD);
    let mut all = Mastery::default();
    for (k, &(i, j)) in jobs.iter().enumerate() {
        out.push_str(&res[k].row(&format!("{} v {}", cs[i].0, cs[j].0)));
        all.merge(&res[k]);
    }
    out.push_str(&all.row("ALL"));
    out
