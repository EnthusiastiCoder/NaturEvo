//! Grace metrics: how smooth, economical and well-timed a fighter's play is. MEASURED ONLY, never rewarded.
use crate::duel::Match;
use crate::moveset::{MoveId, N_MOVES};
use crate::robot::DT;
use crate::skillbot::is_attack;
