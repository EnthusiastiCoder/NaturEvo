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

fn fighter_json(out: &mut String, r: &Robot) {
    let _ = write!(out, "{{\"fx\":{:.2},\"facing\":{},\"parts\":[", r.x, if r.facing >= 0.0 { 1 } else { -1 });
    for (i, p) in Part::ALL.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        cap_json(out, &r.capsule(*p));
    }
    out.push_str("],\"sword\":");
    cap_json(out, &r.sword_capsule());
    out.push_str(",\"shield\":");
    cap_json(out, &r.shield_capsule());
    out.push_str(",\"hp\":[");
    for (i, p) in Part::ALL.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "{}", (r.hp_frac(*p) * 100.0).round() as i32);
    }
    let _ = write!(
        out,
        "],\"energy\":{},\"posture\":{},\"state\":\"{}\"}}",
        r.energy.round() as i32,
        r.posture.round() as i32,
        state_name(r)
    );
}

/// One frame object for the current state of the match.
pub fn frame_json(m: &Match) -> String {
    let mut s = String::with_capacity(2048);
    let _ = write!(s, "{{\"t\":{},\"f\":[", m.tick);
    fighter_json(&mut s, &m.robots[0]);
    s.push(',');
    fighter_json(&mut s, &m.robots[1]);
    s.push_str("]}");
    s
}
