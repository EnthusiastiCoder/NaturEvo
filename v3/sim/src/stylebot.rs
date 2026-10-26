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
