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

/// Required joints, proximal to distal. A disabled/destroyed part of any of these makes the move impossible.
pub fn required_joints(m: Move) -> &'static [Joint] {
    use Joint::*;
    match m {
        Move::Slash | Move::Thrust => &[HipR, Waist, ShoulderR, ElbowR],
        Move::ShieldBash => &[Waist, ShoulderL, ElbowL],
        Move::Guard => &[ShoulderL, ElbowL],
        Move::Parry => &[ShoulderR, ElbowR, Waist],
        Move::KickR => &[HipR, KneeR, AnkleR],
        Move::KickL => &[HipL, KneeL, AnkleL],
        Move::Dodge => &[HipL, HipR, KneeL, KneeR, AnkleL, AnkleR],
    }
}
