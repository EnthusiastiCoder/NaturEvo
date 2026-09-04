//! Fight log: one frame per tick, serialised to JSON by hand (no dependencies).
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
