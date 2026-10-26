//! Scripted style teachers: three archetypes built from the existing move vocabulary, each a function of what a
//! fighter can sense (distance, own and opponent move phase, opening flags), so a net can imitate them.
//! They fight through the same Executor as the net and obey the same rules as every fighter.
//! Press: advances, chains attacks from close range, rarely defends. Counter: holds distance, defends against
//! every telegraph, then ripostes and punishes. Duelist: keeps its optimal spacing, strikes singly with feints
//! and drag, steps out of reach of incoming swings.
use crate::moveset::{MoveId, MoveParams, Phase};
use crate::robot::{Robot, State};
use crate::rng::Rng;
use crate::sense::MoveInfo;
use crate::skillbot::{is_attack, Request};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleId {
    Press,
    Counter,
    Duelist,
}

impl StyleId {
    pub const ALL: [StyleId; 3] = [StyleId::Press, StyleId::Counter, StyleId::Duelist];
    pub fn name(self) -> &'static str {
        match self {
            StyleId::Press => "press",
            StyleId::Counter => "counter",
            StyleId::Duelist => "duelist",
        }
    }
    pub fn from_name(s: &str) -> Option<StyleId> {
        StyleId::ALL.iter().copied().find(|x| x.name() == s)
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

fn p(reach: f32, speed: f32, arc: f32, delay: f32) -> MoveParams {
    MoveParams { reach, speed, arc, delay }
}

const SWING_MAX: f32 = 2.2;
const STUCK_RANGE: f32 = 2.55;
const CLOSE: f32 = 1.35;

/// Per-style tuning: preferred spacing, reaction point and the like.
struct Tune {
    /// Walk toward the opponent beyond this torso distance while it is idle.
    approach: f32,
    /// Back off below this distance while idle (0 = never).
    retreat: f32,
    /// A telegraph is answered once its windup progress passes this.
    react_at: f32,
    /// Strike only from at most this distance.
    strike_max: f32,
}

fn tune(s: StyleId) -> Tune {
    match s {
        StyleId::Press => Tune { approach: 1.75, retreat: 1.1, react_at: 0.75, strike_max: 2.2 },
        StyleId::Counter => Tune { approach: 2.1, retreat: 1.85, react_at: 0.0, strike_max: 2.15 },
        StyleId::Duelist => Tune { approach: 2.2, retreat: 1.95, react_at: 0.15, strike_max: 2.2 },
    }
}

pub struct StyleBot {
    style: StyleId,
    /// Per-fight personality jitter (spacing and reaction), fixed by the seed.
    jitter: [f32; 3],
    last_x: f32,
    stuck: f32,
}

impl StyleBot {
    pub fn new(style: StyleId, seed: u64) -> StyleBot {
        let mut rng = Rng::new(seed ^ 0x57_1E_B07 ^ ((style.index() as u64) << 40));
        let jitter = [rng.range(-0.12, 0.12), rng.range(-0.1, 0.1), rng.range(-0.08, 0.08)];
        StyleBot { style, jitter, last_x: f32::NAN, stuck: 0.0 }
    }

    pub fn style(&self) -> StyleId {
        self.style
    }

    /// The attack for the situation: a pure function of distance and the opponent's state.
    fn pick_attack(&self, dist: f32, theirs: &MoveInfo, opp_open: bool) -> (MoveId, MoveParams) {
        let guard_up = theirs.id == MoveId::Guard && theirs.phase != Phase::Idle;
        let st = self.style;
        if dist < CLOSE {
            return if guard_up { (MoveId::ShieldBash, p(0.8, 0.9, 0.5, 0.0)) } else { (MoveId::Kick, p(0.8, 0.9, 0.5, 0.0)) };
        }
        if guard_up && dist < 1.7 {
            return (MoveId::ShieldBash, p(0.8, 0.9, 0.5, 0.0));
        }
        if opp_open {
            let id = if dist > 1.7 { MoveId::DiagUp } else { MoveId::Thrust };
            return (id, p(0.95, 1.0, 0.5, 0.0));
        }
        if guard_up {
            return (MoveId::LowCut, p(0.9, 0.95, 0.5, 0.0));
        }
        if theirs.id == MoveId::Backstep && theirs.phase != Phase::Idle {
            return (MoveId::Sweep, p(1.0, 1.0, 0.5, 0.0));
        }
        match st {
            StyleId::Press => {
                if dist > 1.95 {
                    (MoveId::DiagUp, p(0.95, 1.0, 0.45, 0.0))
                } else if dist > 1.7 {
                    (MoveId::Overhead, p(0.95, 1.0, 0.45, 0.0))
                } else if dist > 1.5 {
                    (MoveId::DiagDown, p(0.95, 1.0, 0.45, 0.0))
                } else {
                    (MoveId::Thrust, p(0.9, 1.0, 0.45, 0.0))
                }
            }
            StyleId::Counter => {
                if dist > 1.9 {
                    (MoveId::DiagUp, p(0.9, 0.95, 0.45, 0.0))
                } else if dist > 1.6 {
                    (MoveId::Overhead, p(0.9, 0.95, 0.45, 0.0))
                } else {
                    (MoveId::DiagDown, p(0.9, 0.95, 0.45, 0.0))
                }
            }
            StyleId::Duelist => {
                if dist > 1.95 {
                    (MoveId::Thrust, p(1.0, 0.95, 0.5, 0.0))
                } else if dist > 1.7 {
                    (MoveId::DiagUp, p(0.95, 0.95, 0.5, 0.55))
                } else {
                    (MoveId::DiagDown, p(0.9, 0.95, 0.5, 0.55))
                }
            }
        }
    }

    /// Answer to a telegraphed attack, chosen by what is coming.
    fn defend(&self, theirs: &MoveInfo, dist: f32) -> Request {
        if dist > 2.7 {
            return None;
        }
        match theirs.id {
            MoveId::Thrust => {
                if self.style == StyleId::Press {
                    Some((MoveId::Parry, p(0.5, 0.9, 0.5, 0.0)))
                } else {
                    Some((MoveId::Backstep, p(0.8, 0.9, 0.5, 0.0)))
                }
            }
            MoveId::Overhead | MoveId::DiagDown => Some((MoveId::Parry, p(0.5, 0.9, 0.5, 0.0))),
            MoveId::DiagUp | MoveId::LowCut | MoveId::Sweep | MoveId::Kick | MoveId::ShieldBash | MoveId::Riposte => {
                match self.style {
                    StyleId::Press => None,
                    StyleId::Counter => Some((MoveId::Parry, p(0.5, 0.9, 0.5, 0.0))),
                    StyleId::Duelist => Some((MoveId::Guard, p(0.5, 0.8, 0.5, 0.0))),
                }
            }
            _ => None,
        }
    }

    /// One tick: the move request and the footwork drive.
    pub fn act(&mut self, me: &Robot, opp: &Robot, mine: &MoveInfo, theirs: &MoveInfo, _live_threat: bool, dt: f32) -> (Request, f32) {
        let mut t = tune(self.style);
        t.approach += self.jitter[0];
        t.retreat += self.jitter[0] * 0.8;
        t.react_at = (t.react_at + self.jitter[1]).max(0.0);
        t.strike_max += self.jitter[2];
        let dx = opp.x - me.x;
        let dist = dx.abs();
        let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
        let idle = mine.phase == Phase::Idle;
        let down = opp.state() == State::Fallen;
        let stag = opp.state() == State::Staggered;
        let opp_open = theirs.open || stag || down;
        let their_attack = is_attack(theirs.id) && matches!(theirs.phase, Phase::Windup | Phase::Active);
        let guard_up = theirs.id == MoveId::Guard && theirs.phase != Phase::Idle;
        let opp_idle = theirs.phase == Phase::Idle;
