//! Robot body: 13 capsule parts, 12 revolute joints, kinematic FK, rate-limited motors, per-part HP,
//! energy, balance, falling and the stagger meter. Side view: x horizontal (scaled by `facing`), y up.
//!
//! Angle convention: limb angles are measured from "straight down", positive toward the facing
//! direction. Elbow flexes forward (adds to the shoulder angle), knee flexes backward (subtracts from
//! the hip angle), the foot stays flat when hip - knee + ankle... see `fk`.

use crate::config::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Hero,
    Monster,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Hero => "hero",
            Role::Monster => "monster",
        }
    }
}

pub const DT: f32 = PHYS_DT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub r: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Head,
    Torso,
    Pelvis,
    UArmL,
    UArmR,
    FArmL,
    FArmR,
    ThighL,
    ThighR,
    ShinL,
    ShinR,
    FootL,
    FootR,
}

impl Part {
    pub const ALL: [Part; 13] = [
        Part::Head,
        Part::Torso,
        Part::Pelvis,
        Part::UArmL,
        Part::UArmR,
        Part::FArmL,
        Part::FArmR,
        Part::ThighL,
        Part::ThighR,
        Part::ShinL,
        Part::ShinR,
        Part::FootL,
        Part::FootR,
    ];
        }
    }
}
