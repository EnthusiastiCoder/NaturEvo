//! Scripted opponents: a static dummy and an attacker that approaches and swings at random
//! intervals (slash high, slash low, thrust). Deterministic from the match seed.
use crate::robot::{Joint, Robot, STAND_TARGETS};
use crate::rng::Rng;

/// Normalised joint targets plus a drive command.
#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub targets: [f32; 12],
    pub drive: f32,
}

pub fn dummy() -> Action {
    Action { targets: STAND_TARGETS, drive: 0.0 }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Swing {
    SlashHigh,
    SlashLow,
    Thrust,
}

#[derive(Clone, Debug)]
pub struct Scripted {
    rng: Rng,
    swing: Option<(Swing, f32)>,
    wait: f32,
    last_x: f32,
    still: f32,
}

const WINDUP: f32 = 0.3;
const STRIKE: f32 = 0.2;
const RECOVER: f32 = 0.35;
/// Preferred torso distance for swinging, metres.
const REACH: f32 = 2.4;
/// Ticks without progress after which the attacker swings from where it is (an obstacle is in the way).
const STUCK_S: f32 = 0.5;

fn set(t: &mut [f32; 12], j: Joint, v: f32) {
    t[j as usize] = v;
}

fn lerp(a: f32, b: f32, s: f32) -> f32 {
    a + (b - a) * s.clamp(0.0, 1.0)
}

impl Scripted {
    pub fn new(seed: u64) -> Scripted {
        let mut rng = Rng::new(seed ^ 0x5C21_97ED);
        let wait = rng.range(0.4, 1.2);
        Scripted { rng, swing: None, wait, last_x: f32::NAN, still: 0.0 }
    }

    /// Joint targets for one swing at time `t` since it began: (shoulder, elbow, waist, hip) of the sword arm.
    fn pose(kind: Swing, t: f32) -> (f32, f32, f32, f32) {
        let (w, s) = match kind {
            Swing::SlashHigh => ((0.85, 0.2, -0.5, -0.1), (0.05, -0.5, 0.5, 0.0)),
            Swing::SlashLow => ((0.5, 0.1, -0.5, -0.1), (-0.5, -0.6, 0.6, 0.0)),
            Swing::Thrust => ((0.0, 0.8, -0.3, -0.1), (0.2, -0.9, 0.4, 0.0)),
        };
        let rest = (-0.4, 0.0, 0.0, -0.2);
        let lerp4 = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), k: f32| {
            (lerp(a.0, b.0, k), lerp(a.1, b.1, k), lerp(a.2, b.2, k), lerp(a.3, b.3, k))
        };
        if t < WINDUP {
            lerp4(rest, w, t / WINDUP)
        } else if t < WINDUP + STRIKE {
            lerp4(w, s, (t - WINDUP) / STRIKE)
        } else {
            lerp4(s, rest, (t - WINDUP - STRIKE) / RECOVER)
        }
    }

    pub fn act(&mut self, me: &Robot, opp: &Robot, dt: f32) -> Action {
        let mut a = Action { targets: STAND_TARGETS, drive: 0.0 };
        // Ready pose: sword arm lowered so the blade rests clear of the opponent's shield.
        set(&mut a.targets, Joint::ShoulderR, -1.0);
        set(&mut a.targets, Joint::ElbowR, 0.0);
        let dx = opp.x - me.x;
        let dist = dx.abs();
        let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
        if let Some((kind, t)) = self.swing {
            let t = t + dt;
            if t >= WINDUP + STRIKE + RECOVER {
                self.swing = None;
                self.wait = self.rng.range(0.8, 2.0);
            } else {
                self.swing = Some((kind, t));
                let (sh, el, wa, hi) = Self::pose(kind, t);
                set(&mut a.targets, Joint::ShoulderR, sh);
                set(&mut a.targets, Joint::ElbowR, el);
                set(&mut a.targets, Joint::Waist, wa);
                set(&mut a.targets, Joint::HipR, hi);
                return a;
            }
        }
        if (me.x - self.last_x).abs() < 1e-4 {
            self.still += dt;
        } else {
            self.still = 0.0;
        }
        self.last_x = me.x;
        let stuck = self.still > STUCK_S && dist < REACH + 0.7;
        if dist > REACH + 0.2 && !stuck {
            a.drive = dir;
        } else if dist < REACH - 0.8 {
            a.drive = -dir;
        } else {
            self.wait -= dt;
            if self.wait <= 0.0 {
                let kind = match self.rng.next_u64() % 3 {
                    0 => Swing::SlashHigh,
                    1 => Swing::SlashLow,
                    _ => Swing::Thrust,
                };
                self.swing = Some((kind, 0.0));
            }
        }
        a
    }
}
