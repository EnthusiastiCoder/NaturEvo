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

fn add(p: [f32; 2], d: [f32; 2], s: f32) -> [f32; 2] {
    [p[0] + d[0] * s, p[1] + d[1] * s]
}

fn cap_bottom(c: &Capsule) -> f32 {
    c.a[1].min(c.b[1]) - c.r
}

impl Robot {
    pub fn new(role: Role) -> Robot {
        let mut angles = [0.0; 12];
        for j in Joint::ALL {
            angles[j as usize] = j.to_angle(STAND_TARGETS[j as usize]);
        }
        let mut hp = [0.0; 13];
        for p in Part::ALL {
            hp[p as usize] = TABLE[p as usize].1;
        }
        let mut r = Robot {
            role,
            x: 0.0,
            facing: 1.0,
            energy: ENERGY_MAX,
            posture: 0.0,
            sword_ang_vel: 0.0,
            sword_moving_since: 0.0,
            strike_quality: 1.0,
            free_regen_time: 0.0,
            shield_integrity: 100.0,
            hp,
            detached: [false; 13],
            angles,
            caps: [ZERO_CAP; 13],
            sword: ZERO_CAP,
            sword_prev: ZERO_CAP,
            shield: ZERO_CAP,
            debris: vec![None; 13],
            com_off: 0.0,
            com_vel: 0.0,
            neg_margin_time: 0.0,
            since_hit: 10.0,
            stagger_timer: 0.0,
            fallen: false,
            fall_timer: 0.0,
            guard_intent: false,
        };
        r.fk();
        r.sword_prev = r.sword;
        r
    }

    // ---- static part data ----
    pub fn part_mass(&self, p: Part) -> f32 {
        TABLE[p as usize].0
    }
    pub fn max_hp(&self, p: Part) -> f32 {
        TABLE[p as usize].1
    }
    pub fn armour(&self, p: Part) -> f32 {
        TABLE[p as usize].2
    }
    pub fn zone_mult(&self, p: Part) -> f32 {
        TABLE[p as usize].3
    }

    // ---- health ----
    pub fn hp(&self, p: Part) -> f32 {
        self.hp[p as usize]
    }
    /// Health fraction 0..1 (overkill below zero reads as 0).
    pub fn hp_frac(&self, p: Part) -> f32 {
        (self.hp[p as usize] / self.max_hp(p)).clamp(0.0, 1.0)
    }
    /// Test and combat hook: set a part's HP directly, applying the disabled/destroyed thresholds.
    pub fn set_hp(&mut self, p: Part, hp: f32) {
        let floor = -0.5 * self.max_hp(p);
        self.hp[p as usize] = hp.min(self.max_hp(p)).max(floor);
        if self.is_destroyed(p) {
            self.detach(p);
        }
    }
    pub fn is_disabled(&self, p: Part) -> bool {
        self.hp[p as usize] <= 0.0 || self.detached[p as usize]
    }
    pub fn is_destroyed(&self, p: Part) -> bool {
        self.hp[p as usize] <= -0.5 * self.max_hp(p) + 1e-4
    }
    pub fn is_detached(&self, p: Part) -> bool {
        self.detached[p as usize]
    }

    fn detach(&mut self, root: Part) {
        for p in Part::ALL {
            if p.is_under(root) && !self.detached[p as usize] {
                self.detached[p as usize] = true;
                self.debris[p as usize] = Some(Debris { cap: self.caps[p as usize], vy: 0.0 });
            }
        }
    }

    pub fn armour_eff(&self, p: Part) -> f32 {
        self.armour(p) * (0.4 + 0.6 * self.hp_frac(p))
    }

    /// Joint torque scale from part health (and energy). 0 if the part is disabled.
    pub fn torque_scale(&self, j: Joint) -> f32 {
        let p = j.part();
        let mut s = if self.is_disabled(p) { 0.0 } else { (self.hp_frac(p) / 0.5).clamp(0.25, 1.0) };
        if self.energy <= 0.0 {
            s = s.min(EMPTY_ENERGY_TORQUE);
        }
        s
    }

    /// Apply a raw impact to a part: cap, zone multiplier, armour, exposed bonus. Returns hp actually lost.
    pub fn damage_part(&mut self, p: Part, raw_impact: f32) -> f32 {
        if self.detached[p as usize] {
            return 0.0;
        }
        let impact = raw_impact.min(IMPACT_CAP).max(0.0);
        let exposed = if matches!(self.state(), State::Staggered | State::Fallen) { EXPOSED_MULT } else { 1.0 };
        let dmg = impact * self.zone_mult(p) * (1.0 - self.armour_eff(p)) * exposed;
        let before = self.hp[p as usize];
        self.set_hp(p, before - dmg);
        before - self.hp[p as usize]
    }

    // ---- weapon and shield ----
    pub fn has_sword(&self) -> bool {
        !self.detached[Part::FArmR as usize]
    }
    pub fn has_shield(&self) -> bool {
        !self.detached[Part::FArmL as usize] && self.shield_integrity > 0.0
    }
    /// The shield can block: present, intact and powered.
    pub fn shield_active(&self) -> bool {
        self.has_shield() && self.energy > 0.0
    }
    /// Shield arm pose raised in front (shoulder and elbow above thresholds).
    pub fn guard_raised(&self) -> bool {
        self.has_shield()
            && self.angles[Joint::ShoulderL as usize] >= GUARD_SHOULDER
            && self.angles[Joint::ElbowL as usize] >= GUARD_ELBOW
    }
    pub fn sword_capsule(&self) -> Capsule {
        self.sword
    }
    pub fn sword_prev_capsule(&self) -> Capsule {
        self.sword_prev
    }
    pub fn shield_capsule(&self) -> Capsule {
        self.shield
    }
    pub fn set_sword(&mut self, prev: Capsule, cur: Capsule) {
        self.sword_prev = prev;
        self.sword = cur;
    }
    pub fn set_shield(&mut self, cap: Capsule) {
        self.shield = cap;
    }

    // ---- posture / state ----
    pub fn add_posture(&mut self, v: f32) {
        self.since_hit = 0.0;
        if self.stagger_timer > 0.0 {
            return;
        }
        self.posture += v;
        if self.posture >= 100.0 {
            self.stagger_timer = STAGGER_TIME;
            self.posture = STAGGER_RESET;
        }
    }

    pub fn is_ko(&self) -> bool {
        self.is_destroyed(Part::Head) || self.is_destroyed(Part::Torso)
    }

    pub fn state(&self) -> State {
        if self.is_ko() {
            State::Ko
        } else if self.fallen {
            State::Fallen
        } else if self.stagger_timer > 0.0 {
            State::Staggered
        } else {
            State::Idle
        }
    }

    pub fn can_act(&self) -> bool {
        self.state() == State::Idle
    }

    pub fn knock_down(&mut self) {
        if !self.fallen {
            self.fallen = true;
            self.fall_timer = FALL_TIME;
            self.com_off = 0.0;
            self.com_vel = 0.0;
            self.neg_margin_time = 0.0;
        }
    }

    /// Add a horizontal impulse to the CoM velocity (world +x positive).
    pub fn apply_impulse(&mut self, imp: f32) {
        self.com_vel += imp * IMPULSE_TO_VEL;
    }

    pub fn joint_angle(&self, j: Joint) -> f32 {
        self.angles[j as usize]
    }

    // ---- kinematics ----
    pub fn capsule(&self, p: Part) -> Capsule {
        match &self.debris[p as usize] {
            Some(d) => d.cap,
            None => self.caps[p as usize],
        }
    }

    /// Recompute all capsules, sword and shield from joint angles, root x and the ground constraint.
    pub fn fk(&mut self) {
        let f = self.facing;
        let g = |j: Joint| self.angles[j as usize];
        let root = [self.x, 0.0];
        let mut caps = [ZERO_CAP; 13];

        caps[Part::Pelvis as usize] = Capsule { a: root, b: [root[0], root[1] + PELVIS_LEN], r: PELVIS_R };
        let base = [root[0], root[1] + PELVIS_LEN];
        let w = g(Joint::Waist);
        let tdir = [f * w.sin(), w.cos()];
        let neck = add(base, tdir, TORSO_LEN);
        caps[Part::Torso as usize] = Capsule { a: base, b: neck, r: TORSO_R };
        let hang = w + g(Joint::Neck);
        let hdir = [f * hang.sin(), hang.cos()];
        caps[Part::Head as usize] =
            Capsule { a: add(neck, hdir, HEAD_NEAR), b: add(neck, hdir, HEAD_FAR), r: HEAD_R };

        let shoulder = add(base, tdir, SHOULDER_UP);
        let mut hand = [[0.0; 2]; 2];
        let mut fore_ang = [0.0f32; 2];
        let mut fore_start = [[0.0; 2]; 2];
        for (i, (sj, ej, up, fo)) in [
            (Joint::ShoulderL, Joint::ElbowL, Part::UArmL, Part::FArmL),
            (Joint::ShoulderR, Joint::ElbowR, Part::UArmR, Part::FArmR),
        ]
        .into_iter()
        .enumerate()
        {
            let tu = g(sj) + w;
            let elbow = add(shoulder, dir(f, tu), UARM_LEN);
            caps[up as usize] = Capsule { a: shoulder, b: elbow, r: UARM_R };
            let tf = tu + g(ej);
            let h = add(elbow, dir(f, tf), FARM_LEN);
            caps[fo as usize] = Capsule { a: elbow, b: h, r: FARM_R };
            hand[i] = h;
            fore_ang[i] = tf;
            fore_start[i] = elbow;
        }
        // Left forearm carries the shield plate: along the forearm, offset forward of it.
        {
            let tf = fore_ang[0];
            let n = dir(f, tf - std::f32::consts::FRAC_PI_2);
            let c = add(add(fore_start[0], dir(f, tf), 0.5), n, 0.3);
            let d = dir(f, tf);
            self.shield = Capsule { a: add(c, d, -SHIELD_LEN * 0.5), b: add(c, d, SHIELD_LEN * 0.5), r: SHIELD_R };
        }
        // Right forearm carries the sword, pointing forward-up of the forearm.
        {
            let sd = dir(f, fore_ang[1] + std::f32::consts::FRAC_PI_4);
            self.sword = Capsule { a: hand[1], b: add(hand[1], sd, SWORD_LEN), r: SWORD_R };
        }

        for (hj, kj, aj, th, sh, ft) in [
            (Joint::HipL, Joint::KneeL, Joint::AnkleL, Part::ThighL, Part::ShinL, Part::FootL),
            (Joint::HipR, Joint::KneeR, Joint::AnkleR, Part::ThighR, Part::ShinR, Part::FootR),
        ] {
            let th_ang = g(hj);
            let knee = add(root, dir(f, th_ang), THIGH_LEN);
            caps[th as usize] = Capsule { a: root, b: knee, r: THIGH_R };
            let sh_ang = th_ang - g(kj);
            let ankle = add(knee, dir(f, sh_ang), SHIN_LEN);
            caps[sh as usize] = Capsule { a: knee, b: ankle, r: SHIN_R };
            let e = sh_ang + g(aj);
            let fd = [f * e.cos(), e.sin()];
            caps[ft as usize] =
                Capsule { a: add(ankle, fd, -FOOT_BACK), b: add(ankle, fd, FOOT_FRONT), r: FOOT_R };
        }

        // Fallen: rotate the whole body to lie on its back about the root.
        if self.fallen || self.is_ko() {
            let phi = f * std::f32::consts::FRAC_PI_2;
            let (s, c) = phi.sin_cos();
            let rot = |p: [f32; 2]| -> [f32; 2] {
                let (dx, dy) = (p[0] - root[0], p[1] - root[1]);
                [root[0] + dx * c - dy * s, root[1] + dx * s + dy * c]
            };
            for cp in caps.iter_mut() {
                cp.a = rot(cp.a);
                cp.b = rot(cp.b);
            }
            self.sword.a = rot(self.sword.a);
            self.sword.b = rot(self.sword.b);
            self.shield.a = rot(self.shield.a);
            self.shield.b = rot(self.shield.b);
        }

        // Ground solve: lowest attached part touches y = 0.
        let mut low = f32::MAX;
        for p in Part::ALL {
            if !self.detached[p as usize] {
                low = low.min(cap_bottom(&caps[p as usize]));
            }
        }
        if low == f32::MAX {
            low = 0.0;
        }
        for cp in caps.iter_mut() {
            cp.a[1] -= low;
            cp.b[1] -= low;
        }
        for cp in [&mut self.sword, &mut self.shield] {
            cp.a[1] -= low;
            cp.b[1] -= low;
        }
        self.caps = caps;
    }

    // ---- mass, CoM, balance ----
    pub fn total_mass(&self) -> f32 {
        Part::ALL.iter().filter(|p| !self.detached[**p as usize]).map(|p| self.part_mass(*p)).sum()
    }

    /// Mass-weighted centre of the surviving parts (plus the balance offset in x).
    pub fn com(&self) -> [f32; 2] {
        let (mut sx, mut sy, mut m) = (0.0, 0.0, 0.0);
        for p in Part::ALL {
            if self.detached[p as usize] {
                continue;
            }
            let c = &self.caps[p as usize];
            let pm = self.part_mass(p);
            sx += pm * (c.a[0] + c.b[0]) * 0.5;
            sy += pm * (c.a[1] + c.b[1]) * 0.5;
            m += pm;
        }
        if m <= 0.0 {
            return [self.x, 0.0];
        }
        [sx / m + self.com_off, sy / m]
    }

    /// Distance of the CoM x inside the grounded-foot span (negative outside).
    pub fn balance_margin(&self) -> f32 {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for p in [Part::FootL, Part::FootR] {
            if self.detached[p as usize] {
                continue;
            }
            let c = &self.caps[p as usize];
            if cap_bottom(c) <= 0.08 {
                lo = lo.min(c.a[0].min(c.b[0]) - c.r);
                hi = hi.max(c.a[0].max(c.b[0]) + c.r);
            }
        }
        if lo > hi {
            return -1.0;
        }
        let cx = self.com()[0];
        let mut m = (cx - lo).min(hi - cx);
        for p in [Part::ThighL, Part::ThighR, Part::ShinL, Part::ShinR] {
            if self.is_disabled(p) {
                m -= 0.5;
            }
        }
        m
    }

    fn can_get_up(&self) -> bool {
        use Part::*;
        if Part::ALL.iter().any(|p| p.is_leg() && self.is_destroyed(*p)) {
            return false;
        }
        !self.is_disabled(Pelvis)
            && !self.is_disabled(ThighL)
            && !self.is_disabled(ThighR)
            && !(self.is_disabled(ShinL) && self.is_disabled(ShinR))
    }

    /// Walk: `drive` in -1..1 (world x), scaled by leg torque, clamped to the arena. Costs energy.
    pub fn apply_drive(&mut self, drive: f32, dt: f32, arena_width: f32) {
        if !self.can_act() {
            return;
        }
        let legs = [Joint::HipL, Joint::HipR, Joint::KneeL, Joint::KneeR];
        let scale = legs.iter().map(|j| self.torque_scale(*j)).sum::<f32>() / 4.0;
        let d = drive.clamp(-1.0, 1.0);
        self.x = (self.x + d * DRIVE_SPEED * scale * dt).clamp(0.5, arena_width - 0.5);
        self.energy = (self.energy - d.abs() * 1.0 * dt).max(0.0);
        self.fk();
    }
        }
    }
}
