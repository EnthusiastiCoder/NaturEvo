//! Fight log: one frame per tick, serialised to JSON by hand (no dependencies).
use crate::attacks::{Rank, Species};
use crate::fighter::{Action, CombatState};
use std::fmt::Write;

/// Something that happened this tick (hits, damage over time, parries, wall slams, guard breaks).
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Hit { by: usize, atk: usize, dmg: f32, blocked: bool, punish: bool, fx: Vec<usize> },
    Dot { who: usize, kind: &'static str, dmg: f32 },
    Parry { by: usize },
    Slam { who: usize, dmg: f32 },
    GuardBreak { who: usize },
}

impl Event {
    fn write(&self, s: &mut String) {
        match self {
            Event::Hit { by, atk, dmg, blocked, punish, fx } => {
                let _ = write!(s, "{{\"t\":\"hit\",\"by\":{},\"atk\":{},\"dmg\":{},\"blocked\":{},\"punish\":{},\"fx\":[", by, atk, dmg, blocked, punish);
                for (i, k) in fx.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    let _ = write!(s, "{}", k);
                }
                s.push_str("]}");
            }
            Event::Dot { who, kind, dmg } => {
                let _ = write!(s, "{{\"t\":\"dot\",\"who\":{},\"kind\":\"{}\",\"dmg\":{}}}", who, kind, dmg);
            }
            Event::Parry { by } => {
                let _ = write!(s, "{{\"t\":\"parry\",\"by\":{}}}", by);
            }
            Event::Slam { who, dmg } => {
                let _ = write!(s, "{{\"t\":\"slam\",\"who\":{},\"dmg\":{}}}", who, dmg);
            }
            Event::GuardBreak { who } => {
                let _ = write!(s, "{{\"t\":\"guard_break\",\"who\":{}}}", who);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub pos: [(f32, f32); 2],
    pub facing: [f32; 2],
    pub hp: [f32; 2],
    pub stamina: [f32; 2],
    pub action: [Action; 2],
    pub state: [CombatState; 2],
    /// Move index being swung (Windup/Active/Recovery), else -1.
    pub atk: [i8; 2],
    /// Remaining ticks per status, indexed by `Status::idx`.
    pub fx: [[u32; 7]; 2],
    /// Whether each fighter currently sees the other.
    pub seen: [bool; 2],
    pub ev: Vec<Event>,
}

#[derive(Clone, Debug)]
pub struct FightLog {
    pub tick_hz: u32,
    pub frames: Vec<Frame>,
    /// Optional species and rank of each side, written to the JSON header.
    pub meta: Option<([Species; 2], [Rank; 2])>,
}

impl FightLog {
    pub fn new(tick_hz: u32) -> FightLog {
        FightLog { tick_hz, frames: Vec::new(), meta: None }
    }

    pub fn with_capacity(tick_hz: u32, n: usize) -> FightLog {
        FightLog { tick_hz, frames: Vec::with_capacity(n) }
    }

    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(self.frames.len() * 300 + 64);
        let _ = write!(s, "{{\"tick_hz\":{},\"frames\":[", self.tick_hz);
        for (i, f) in self.frames.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(
                s,
                "{{\"pos\":[[{},{}],[{},{}]],\"facing\":[{},{}],\"hp\":[{},{}],\"stamina\":[{},{}],\"action\":[",
                f.pos[0].0, f.pos[0].1, f.pos[1].0, f.pos[1].1, f.facing[0], f.facing[1], f.hp[0], f.hp[1], f.stamina[0], f.stamina[1]
            );
            for (k, a) in f.action.iter().enumerate() {
                if k > 0 {
                    s.push(',');
                }
                let _ = write!(
                    s,
                    "{{\"fwd\":{},\"strafe\":{},\"turn\":{},\"attack\":{},\"block\":{},\"dodge\":{}}}",
                    a.fwd, a.strafe, a.turn, a.attack, a.block, a.dodge
                );
            }
            let _ = write!(s, "],\"state\":[\"{}\",\"{}\"]}}", f.state[0].as_str(), f.state[1].as_str());
        }
        s.push_str("]}");
        s
    }
}
