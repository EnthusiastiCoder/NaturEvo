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

    /// Angle (rad) the joint relaxes to when its part is disabled or the robot is down.
    fn relaxed(&self) -> f32 {
        use Joint::*;
        match self {
            ElbowL | ElbowR => 0.3,
            _ => 0.0,
        }
    }

    fn to_angle(&self, t: f32) -> f32 {
        let (lo, hi) = self.limits();
        lo + (t.clamp(-1.0, 1.0) + 1.0) * 0.5 * (hi - lo)
    }
}

// Normalised targets (-1..1 over the joint range).
// Order: Neck, Waist, ShoulderL, ShoulderR, ElbowL, ElbowR, HipL, HipR, KneeL, KneeR, AnkleL, AnkleR.
pub const STAND_TARGETS: [f32; 12] = [0.0, 0.0, -0.4, -0.4, 0.0, 0.0, -0.2, -0.2, -1.0, -1.0, 0.0, 0.0];
/// Left (shield) arm raised: shoulder 80 deg, elbow ~90 deg, forearm nearly vertical in front of the head.
pub const GUARD_TARGETS: [f32; 12] = [0.0, 0.0, 0.1, -0.4, 0.29, 0.0, -0.2, -0.2, -1.0, -1.0, 0.0, 0.0];
/// Staggered recoil: lean back, head back, arms flung out.
pub const RECOIL_TARGETS: [f32; 12] = [-0.6, -0.8, -0.8, -0.8, -0.3, -0.3, -0.2, -0.2, -1.0, -1.0, 0.0, 0.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Staggered,
    Fallen,
    Ko,
}

// Per-part table: (mass kg, max hp, armour, zone multiplier).
const TABLE: [(f32, f32, f32, f32); 13] = [
    (40.0, 60.0, 0.30, 1.6),   // Head
    (300.0, 220.0, 0.45, 1.0), // Torso
    (180.0, 140.0, 0.40, 1.0), // Pelvis
    (70.0, 80.0, 0.30, 0.8),   // UArmL
    (70.0, 80.0, 0.30, 0.8),   // UArmR
    (50.0, 70.0, 0.30, 1.2),   // FArmL
    (50.0, 70.0, 0.30, 1.2),   // FArmR
    (120.0, 110.0, 0.35, 0.9), // ThighL
    (120.0, 110.0, 0.35, 0.9), // ThighR
    (90.0, 90.0, 0.30, 1.2),   // ShinL
    (90.0, 90.0, 0.30, 1.2),   // ShinR
    (40.0, 60.0, 0.30, 0.6),   // FootL
    (40.0, 60.0, 0.30, 0.6),   // FootR
];

// Geometry (metres).
const PELVIS_LEN: f32 = 0.3;
const PELVIS_R: f32 = 0.25;
const TORSO_LEN: f32 = 1.0;
const TORSO_R: f32 = 0.3;
const SHOULDER_UP: f32 = 0.8; // along torso from its base
const HEAD_NEAR: f32 = 0.25;
const HEAD_FAR: f32 = 0.4;
const HEAD_R: f32 = 0.25;
const UARM_LEN: f32 = 0.8;
const UARM_R: f32 = 0.14;
const FARM_LEN: f32 = 0.8;
const FARM_R: f32 = 0.12;
const THIGH_LEN: f32 = 0.9;
const THIGH_R: f32 = 0.17;
const SHIN_LEN: f32 = 0.9;
const SHIN_R: f32 = 0.12;
const FOOT_BACK: f32 = 0.15;
const FOOT_FRONT: f32 = 0.35;
const FOOT_R: f32 = 0.1;
const SWORD_LEN: f32 = 2.2;
const SWORD_R: f32 = 0.05;
const SHIELD_LEN: f32 = 1.2;
const SHIELD_R: f32 = 0.05;

const GUARD_SHOULDER: f32 = 60.0 * std::f32::consts::PI / 180.0;
const GUARD_ELBOW: f32 = 50.0 * std::f32::consts::PI / 180.0;
/// Impulse (N s equivalent, test and combat unit) to CoM velocity (m/s).
const IMPULSE_TO_VEL: f32 = 0.5;
// Inverted-pendulum recovery: spring, damping (about zeta 0.7) and falling rate.
const BAL_K: f32 = 20.0;
const BAL_C: f32 = 6.3;
const BAL_FALL: f32 = 10.0;

#[derive(Clone, Debug)]
struct Debris {
    cap: Capsule,
    vy: f32,
}

#[derive(Clone, Debug)]
pub struct Robot {
    pub role: Role,
    pub x: f32,
    pub facing: f32,
    pub energy: f32,
    pub posture: f32,
    pub sword_ang_vel: f32,
    pub sword_moving_since: f32,
    pub strike_quality: f32,
    pub free_regen_time: f32,
    pub shield_integrity: f32,
    hp: [f32; 13],
    detached: [bool; 13],
    angles: [f32; 12],
    caps: [Capsule; 13],
    sword: Capsule,
    sword_prev: Capsule,
    shield: Capsule,
    debris: Vec<Option<Debris>>,
    com_off: f32,
    com_vel: f32,
    neg_margin_time: f32,
    since_hit: f32,
    stagger_timer: f32,
    fallen: bool,
    fall_timer: f32,
    guard_intent: bool,
}

const ZERO_CAP: Capsule = Capsule { a: [0.0, 0.0], b: [0.0, 0.0], r: 0.0 };

fn dir(f: f32, theta: f32) -> [f32; 2] {
    [f * theta.sin(), -theta.cos()]
}
        }
    }
}
