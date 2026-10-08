//! Fight log (plan section 6): hand-rolled JSON consumed by the viewer. Frames every 2 physics ticks,
//! capsule endpoints in integer centimetres, events with point, direction and impact.
use crate::duel::{Match, Outcome};
use crate::robot::{Capsule, Part, Robot, State};
use std::fmt::Write;

/// Recorded frames and events (pre-rendered JSON fragments).
#[derive(Clone, Debug, Default)]
pub struct FightLog {
    pub frames: Vec<String>,
    pub events: Vec<String>,
}

fn cm(v: f32) -> i32 {
    if v.is_finite() {
        (v * 100.0).round() as i32
    } else {
        0
    }
}

fn cap_json(out: &mut String, c: &Capsule) {
    let _ = write!(out, "[{},{},{},{}]", cm(c.a[0]), cm(c.a[1]), cm(c.b[0]), cm(c.b[1]));
}

fn state_name(r: &Robot) -> &'static str {
    match r.state() {
        State::Idle => "idle",
        State::Staggered => "staggered",
        State::Fallen => "fallen",
        State::Ko => "ko",
    }
}
