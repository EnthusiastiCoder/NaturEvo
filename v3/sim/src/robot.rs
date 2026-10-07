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

    pub fn name(&self) -> &'static str {
        ["Head", "Torso", "Pelvis", "UArmL", "UArmR", "FArmL", "FArmR", "ThighL", "ThighR", "ShinL", "ShinR", "FootL", "FootR"]
            [*self as usize]
    }

    /// Kinematic parent (None for the pelvis root).
    pub fn parent(&self) -> Option<Part> {
        use Part::*;
        match self {
            Pelvis => None,
            Torso | ThighL | ThighR => Some(Pelvis),
            Head | UArmL | UArmR => Some(Torso),
            FArmL => Some(UArmL),
            FArmR => Some(UArmR),
            ShinL => Some(ThighL),
            ShinR => Some(ThighR),
            FootL => Some(ShinL),
            FootR => Some(ShinR),
        }
    }

    /// True if `self` is `anc` or below it in the chain.
    pub fn is_under(&self, anc: Part) -> bool {
        let mut p = Some(*self);
        while let Some(c) = p {
            if c == anc {
                return true;
            }
            p = c.parent();
        }
        false
    }

    fn is_leg(&self) -> bool {
        use Part::*;
        matches!(self, ThighL | ThighR | ShinL | ShinR | FootL | FootR)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Joint {
    Neck,
    Waist,
    ShoulderL,
    ShoulderR,
    ElbowL,
    ElbowR,
    HipL,
    HipR,
    KneeL,
    KneeR,
    AnkleL,
    AnkleR,
}

impl Joint {
    pub const ALL: [Joint; 12] = [
        Joint::Neck,
        Joint::Waist,
        Joint::ShoulderL,
        Joint::ShoulderR,
        Joint::ElbowL,
        Joint::ElbowR,
        Joint::HipL,
        Joint::HipR,
        Joint::KneeL,
        Joint::KneeR,
        Joint::AnkleL,
        Joint::AnkleR,
    ];

    /// Joint range in radians.
    pub fn limits(&self) -> (f32, f32) {
        use Joint::*;
        let (lo, hi): (f32, f32) = match self {
            Neck => (-35.0, 35.0),
            Waist => (-30.0, 30.0),
            ShoulderL | ShoulderR => (-30.0, 170.0),
            ElbowL | ElbowR => (0.0, 140.0),
            HipL | HipR => (-60.0, 90.0),
            KneeL | KneeR => (0.0, 130.0),
            AnkleL | AnkleR => (-30.0, 30.0),
        };
        (lo.to_radians(), hi.to_radians())
    }

    /// The part this joint moves (the distal part).
    pub fn part(&self) -> Part {
        use Joint::*;
        match self {
            Neck => Part::Head,
            Waist => Part::Torso,
            ShoulderL => Part::UArmL,
            ShoulderR => Part::UArmR,
            ElbowL => Part::FArmL,
            ElbowR => Part::FArmR,
            HipL => Part::ThighL,
            HipR => Part::ThighR,
            KneeL => Part::ShinL,
            KneeR => Part::ShinR,
            AnkleL => Part::FootL,
            AnkleR => Part::FootR,
        }
    }

    /// Nominal maximum angular velocity, rad/s.
    pub fn max_speed(&self) -> f32 {
        use Joint::*;
        match self {
            Neck => 6.0,
            Waist => 5.0,
            ShoulderL | ShoulderR | ElbowL | ElbowR => 9.0,
            HipL | HipR => 5.0,
            KneeL | KneeR => 6.0,
            AnkleL | AnkleR => 5.0,
        }
    }
        }
    }
}
