//! Style evaluation: pair fighters (teachers or trained champions), run fights over fixed seeds and report win
//! rates and behavioural metrics per style. Measurement only; nothing here feeds back into fitness.
use crate::arena::Arena;
use crate::duel::{Brain, Match, MatchResult, Outcome};
use crate::grace::match_grace;
use crate::moveset::N_MOVES;
use crate::net::N_OUT;

/// Behaviour totals of one side across fights (divide by `n` for means).
#[derive(Clone, Debug, Default)]
pub struct SideAcc {
    pub n: u32,
    pub wins: u32,
    pub draws: u32,
    pub capped: u32,
    pub ticks: f64,
    pub distinct: f64,
    pub union_moves: [bool; N_MOVES],
    pub gap_mean: f64,
    pub engaged_gap: f64,
    pub spacing_std: f64,
    pub hits_per_swing: f64,
    pub whiff: f64,
    pub punish: f64,
    pub parry_hits: f64,
    pub defences: f64,
    pub parry_defs: f64,
    pub combo_mean: f64,
    pub combo_max: f64,
    pub jerk: f64,
