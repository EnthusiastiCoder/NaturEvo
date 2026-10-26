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
