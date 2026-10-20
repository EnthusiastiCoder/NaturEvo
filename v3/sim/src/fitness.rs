//! Fitness. There is no block bit and no guard bonus: defence pays only through per-threat credit.
use crate::config::{STALL_GRACE_TICKS, STALL_REF_TICKS, W_STALL, W_STALL_MAX, CAPPED_SCORE, DRAW_SCORE, IMPACT_REF};
use crate::duel::{MatchResult, Outcome, Resolution};
use crate::robot::Part;

pub const W_WIN: f32 = 4.0;
pub const W_DEALT: f32 = 3.0;
pub const W_TAKEN: f32 = 2.5;
pub const W_DEFENCE: f32 = 2.0;
pub const W_IDLE_GUARD: f32 = 1.0;
pub const W_PASSIVITY: f32 = 1.0;
pub const W_CHAIN: f32 = 0.5;
/// Small cost on jerky joint motion (mean |dv|/vmax per tick); typical net values 0.1, calm 0.01.
pub const W_JERK: f32 = 8.0;
pub const KO_BONUS: f32 = 1.0;
pub const MAX_CREDITED_THREATS: usize = 6;

#[derive(Clone, Copy, Debug, Default)]
pub struct MatchStats {
    pub win: bool,
    pub draw: bool,
    /// The emergency cap ended the fight: a mutual loss, scored below everything else.
    pub capped: bool,
    /// Summed stall units over the fight's no-hit stretches (see `stall_units`).
    pub stall: f32,
    pub ko: bool,
    pub dmg_dealt_norm: f32,
    pub dmg_taken_norm: f32,
    pub defence_credit: f32,
    pub idle_guard: f32,
    pub passivity: f32,
    pub chain_quality: f32,
    pub jerk: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThreatOutcome {
    pub resolution: Resolution,
    pub i_potential: f32,
    pub followup_hit: bool,
}

/// Zone weight of a part in damage dealt: head 1.5, core 1.0, limbs 0.7.
pub fn zone_weight(p: Part) -> f32 {
    match p {
        Part::Head => 1.5,
        Part::Torso | Part::Pelvis => 1.0,
        _ => 0.7,
    }
}

/// Raw sum of per-threat credits; each threat paid once, at most 6 threats, halved after a follow-up hit.
pub fn defence_credit(threats: &[ThreatOutcome]) -> f32 {
    let mut sum = 0.0;
    let mut n = 0;
    for t in threats {
        let base = match t.resolution {
            Resolution::Parried => (t.i_potential / IMPACT_REF).clamp(0.0, 1.0),
            Resolution::Blocked | Resolution::Dodged => 0.7,
            Resolution::Hit | Resolution::Miss => 0.0,
        };
        if base <= 0.0 || n >= MAX_CREDITED_THREATS {
            continue;
        }
        n += 1;
        sum += if t.followup_hit { base * 0.5 } else { base };
    }
    sum
}

/// Stall units of one no-hit stretch of `len` ticks: free up to the grace, then superlinear.
pub fn stall_units(len: u32) -> f32 {
    if len <= STALL_GRACE_TICKS {
        return 0.0;
    }
    ((len - STALL_GRACE_TICKS) as f32 / STALL_REF_TICKS).powf(1.5)
}

pub fn score_stats(s: &MatchStats) -> f32 {
    if s.capped {
        // Mutual loss: far below any death-loss (>= -6.1) and a draw (-6); damage keeps a small gradient.
        return CAPPED_SCORE + W_DEALT * s.dmg_dealt_norm - W_TAKEN * s.dmg_taken_norm;
    }
    if s.draw {
        return DRAW_SCORE;
    }
    let mut v = W_DEALT * s.dmg_dealt_norm - W_TAKEN * s.dmg_taken_norm + W_DEFENCE * s.defence_credit
        - W_IDLE_GUARD * s.idle_guard
        - W_PASSIVITY * s.passivity
        - W_JERK * s.jerk
        - (W_STALL * s.stall).min(W_STALL_MAX)
        + W_CHAIN * s.chain_quality;
    if s.win {
        v += W_WIN;
    }
    if s.ko {
        v += KO_BONUS;
    }
    v
}

pub fn stats_for(r: &MatchResult, who: usize) -> MatchStats {
    let mine = if who == 0 { Outcome::WinA } else { Outcome::WinB };
    MatchStats {
        win: r.outcome == mine,
        draw: r.outcome == Outcome::Draw,
        capped: r.capped,
        stall: r.stall,
        ko: r.outcome == mine && r.ko[1 - who],
        dmg_dealt_norm: r.dmg_dealt_norm[who],
        dmg_taken_norm: r.dmg_dealt_norm[1 - who].min(1.0),
        defence_credit: r.defence_credit[who],
        idle_guard: r.idle_guard_share[who],
        passivity: r.passivity,
        chain_quality: r.chain_quality[who],
        jerk: r.jerk[who],
    }
}

pub fn score_match(r: &MatchResult, who: usize) -> f32 {
    score_stats(&stats_for(r, who))
}
