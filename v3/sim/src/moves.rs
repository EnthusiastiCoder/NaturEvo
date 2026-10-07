//! Move table, strike quality formula and move requirements.
//! Controllers output joint targets directly; moves are recognised post hoc.
use crate::config::IMPACT_CAP;
use crate::robot::{Joint, Part, Robot, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Slash,
    Thrust,
    ShieldBash,
    KickR,
    KickL,
    Guard,
    Parry,
    Dodge,
}
