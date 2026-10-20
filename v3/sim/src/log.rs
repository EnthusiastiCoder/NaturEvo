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

pub fn f2(v: f32) -> String {
    if v.is_finite() {
        format!("{:.2}", v)
    } else {
        "0.00".to_string()
    }
}

pub fn outcome_name(o: Outcome) -> &'static str {
    match o {
        Outcome::WinA => "win_a",
        Outcome::WinB => "win_b",
        Outcome::Draw => "draw",
    }
}

/// The "geometry" block: the single source of truth for what the viewer draws (integer cm).
pub fn geometry_json(s: &mut String) {
    use crate::config::*;
    let r = Robot::new(crate::robot::Role::Hero);
    let _ = write!(
        s,
        "\"geometry\":{{\"head_r\":{},\"neck_r\":{},\"neck\":\"torso_top_to_head_centre,part_of_head_zone\",\"shield_len\":{},\"shield_along\":{},\"shield_r\":{},\"shield_orient\":\"perpendicular\",\"sword_len\":{},\"sword_r\":{},\"radii\":[",
        cm(HEAD_R), cm(NECK_R), cm(SHIELD_LEN), cm(SHIELD_ALONG), cm(SHIELD_R), cm(SWORD_LEN), cm(SWORD_R)
    );
    for (i, p) in Part::ALL.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "{}", cm(r.capsule(*p).r));
    }
    s.push_str("]}");
}

/// Full fight JSON for a match run with `enable_log()`.
pub fn fight_json(m: &Match) -> String {
    let empty = FightLog::default();
    let log = m.log.as_ref().unwrap_or(&empty);
    let mut s = String::with_capacity(log.frames.len() * 1100 + 4096);
    let _ = write!(s, "{{\"version\":3,\"arena\":{{\"width\":{},\"height\":6,\"obstacles\":[", m.arena.width);
    for (i, o) in m.arena.obstacles.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "{{\"x\":{},\"w\":{},\"h\":{}}}", f2(o.x), f2(o.w), f2(o.h));
    }
    let _ = write!(s, "],\"biome\":\"{}\"}},\"parts\":[", m.arena.biome.name());
    for (i, p) in Part::ALL.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "\"{}\"", p.name());
    }
    s.push_str("],");
    geometry_json(&mut s);
    s.push_str(",\"dt\":0.0333,\"frames\":[");
    s.push_str(&log.frames.join(","));
    s.push_str("],\"events\":[");
    s.push_str(&log.events.join(","));
    let r = m.result();
    let _ = write!(s, "],\"result\":{{\"outcome\":\"{}\",\"ticks\":{},\"capped\":{},\"defence\":[", outcome_name(r.outcome), r.ticks, r.capped);
    for (i, d) in r.defence.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            "{{\"parry\":{},\"block\":{},\"dodge\":{},\"hit\":{},\"miss\":{}}}",
            d.parry, d.block, d.dodge, d.hit, d.miss
        );
    }
    s.push_str("]}}");
    s
}
