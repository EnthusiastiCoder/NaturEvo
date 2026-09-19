//! Fight log: one frame per tick, serialised to JSON by hand (no dependencies).
use crate::attacks::{Rank, Species};
use crate::fighter::{Action, CombatState};
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct Frame {
    pub pos: [(f32, f32); 2],
    pub facing: [f32; 2],
    pub hp: [f32; 2],
    pub stamina: [f32; 2],
    pub action: [Action; 2],
    pub state: [CombatState; 2],
}

#[derive(Clone, Debug)]
pub struct FightLog {
    pub tick_hz: u32,
    pub frames: Vec<Frame>,
}

impl FightLog {
    pub fn new(tick_hz: u32) -> FightLog {
        FightLog { tick_hz, frames: Vec::new() }
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
