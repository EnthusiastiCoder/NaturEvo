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
/// Credit for hits per swing (attack moves started, smoothed), for hits on punish windows, and per distinct attack
/// move that actually landed (saturates at `VARIETY_MAX`). Never paid for guarding, blocking or parrying.
pub const W_ACCURACY: f32 = 2.0;
pub const W_PUNISH: f32 = 2.0;
pub const W_VARIETY: f32 = 0.6;
pub const VARIETY_MAX: u32 = 4;
/// Live threats a fighter must create to earn full pressure credit in a capped fight.
pub const PRESSURE_REF: f32 = 8.0;
/// Capped fights: the side that threatened the other keeps up to this much of the mutual loss back (stays below any death-loss).
pub const W_PRESSURE_CAPPED: f32 = 2.5;
/// Extra KO bonus for a fast finish (scaled by the unused share of the hard cap).
pub const KO_SPEED_BONUS: f32 = 1.5;
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
    /// Live threats this fighter created, over `PRESSURE_REF`, clamped 0..1.
    pub pressure: f32,
    /// Body hits per attack swing, smoothed: hits / (swings + 2).
    pub accuracy: f32,
    /// Hits landed on punish windows over the opponent's windows (0..1).
    pub punish: f32,
    /// Distinct attack moves that landed a hit.
    pub landed_moves: u32,
    /// Fight length over the hard cap, 0..1.
    pub ticks_frac: f32,
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
        return CAPPED_SCORE + W_DEALT * s.dmg_dealt_norm - W_TAKEN * s.dmg_taken_norm + W_PRESSURE_CAPPED * s.pressure.clamp(0.0, 1.0);
    }
    if s.draw {
        return DRAW_SCORE;
    }
    let mut v = W_DEALT * s.dmg_dealt_norm - W_TAKEN * s.dmg_taken_norm + W_DEFENCE * s.defence_credit
        - W_IDLE_GUARD * s.idle_guard
        - W_PASSIVITY * s.passivity
        - W_JERK * s.jerk
        - (W_STALL * s.stall).min(W_STALL_MAX)
        + W_CHAIN * s.chain_quality
        + W_ACCURACY * s.accuracy
        + W_PUNISH * s.punish
        + W_VARIETY * s.landed_moves.min(VARIETY_MAX) as f32;
    if s.win {
        v += W_WIN;
    }
    if s.ko {
        v += KO_BONUS + KO_SPEED_BONUS * (1.0 - s.ticks_frac).clamp(0.0, 1.0);
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
        pressure: {
            let d = &r.defence[1 - who];
            ((d.parry + d.block + d.dodge + d.hit + d.miss) as f32 / PRESSURE_REF).min(1.0)
        },
        accuracy: r.hits[who] as f32 / (r.swings[who] as f32 + 2.0),
        punish: if r.open_windows[1 - who] > 0 { (r.punish_hits[who] as f32 / r.open_windows[1 - who] as f32).min(1.0) } else { 0.0 },
        landed_moves: r.landed_moves[who],
        ticks_frac: (r.ticks as f32 / crate::config::HARD_CAP_TICKS as f32).min(1.0),
    }
}

pub fn score_match(r: &MatchResult, who: usize) -> f32 {
    score_stats(&stats_for(r, who))
}

/// Normalised Shannon entropy (0..1) of a move-count histogram over the 8 attack kinds a style can choose between:
/// 0 when one move is all there is, 1 when the share is even across all of them.
pub fn move_entropy(counts: &[u16; 14]) -> f32 {
    let total: f32 = counts.iter().map(|&c| c as f32).sum();
    if total < 1.0 {
        return 0.0;
    }
    let h: f32 = counts.iter().filter(|&&c| c > 0).map(|&c| {
        let p = c as f32 / total;
        -p * p.ln()
    }).sum();
    (h / 8.0f32.ln()).clamp(0.0, 1.0)
}
