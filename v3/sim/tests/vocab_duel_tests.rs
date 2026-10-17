// WS-C duel integration tests for the vocabulary (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-C).
//
// INVENTED API (see skillbot_tests.rs for Brain::Skill, Brain::Spam, Brain::Cycle and Match::move_starts):
//   Log contract (additive, version stays 3), produced by log.rs when Match::enable_log() is on:
//     every fighter object in every frame gains `"mv":{"id":"Overhead","phase":0..3,"p":0..1,"open":bool,"gb":bool,"rip":bool}`
//       (phase: 0 Idle, 1 Windup, 2 Active, 3 Recovery);
//     events: {"t":..,"kind":"move","who":W,"name":"..","reach":..,"speed":..,"arc":..,"delay":..}
//             {"kind":"whiff","who","name"}  {"kind":"punish","who","victim","name","mult"}  {"kind":"guard_break","who","victim"}
//             {"kind":"feint","who"}  {"kind":"counter","who","name"}  {"kind":"riposte_open","who"}
//   Fighters controlled by Brain::Net execute through the Executor (net outputs decode to a move plus params).
use mech_sim::config::HARD_CAP_TICKS;
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::log::fight_json;
use mech_sim::moveset::MoveId;
use mech_sim::rng::Rng;
use mech_sim::robot::{Part, Role};

fn logged(a: Brain, b: Brain, seed: u64) -> String {
    let mut m = Match::new(seed, a, b);
    m.enable_log();
    m.run_to_end();
    fight_json(&m)
}

/// Every `{...}` event object of the given kind (events are flat objects).
fn events_of<'a>(j: &'a str, kind: &str) -> Vec<&'a str> {
    let key = format!("\"kind\":\"{}\"", kind);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = j[from..].find(&key) {
        let at = from + i;
        let start = j[..at].rfind('{').unwrap();
        let end = at + j[at..].find('}').unwrap();
        out.push(&j[start..=end]);
        from = end;
    }
    out
}

#[test]
fn every_frame_fighter_has_an_mv_object_with_id_phase_p_open() {
    let j = logged(Brain::Skill(3), Brain::Skill(2), 3);
    let a = j.find("\"frames\":[").expect("frames");
    let b = j.find("\"events\":[").expect("events");
    let frames = &j[a..b];
    let n_frames = frames.matches("{\"t\":").count();
    assert!(n_frames > 10);
    assert_eq!(frames.matches("\"mv\":{").count(), 2 * n_frames, "one mv per fighter per frame");
    for key in ["\"id\":\"", "\"phase\":", "\"p\":", "\"open\":", "\"gb\":", "\"rip\":"] {
        assert!(frames.matches(key).count() >= 2 * n_frames, "mv key {} missing", key);
    }
    // At least one non-Idle id appears in a fight between two attackers.
    assert!(["Overhead", "DiagDown", "DiagUp", "Sweep", "Thrust", "LowCut", "Parry", "Guard"].iter().any(|n| frames.contains(&format!("\"id\":\"{}\"", n))));
}

#[test]
fn mv_ids_are_known_move_names_and_phase_in_0_to_3() {
    let j = logged(Brain::Skill(3), Brain::Skill(2), 4);
    let known = ["Idle", "Overhead", "DiagDown", "DiagUp", "Sweep", "Thrust", "LowCut", "Feint", "Parry", "Guard", "Riposte", "ShieldBash", "Kick", "Backstep"];
    let mut from = 0;
    while let Some(i) = j[from..].find("\"mv\":{\"id\":\"") {
        let s = from + i + "\"mv\":{\"id\":\"".len();
        let name = &j[s..s + j[s..].find('"').unwrap()];
        assert!(known.contains(&name), "unknown id {}", name);
        let rest = &j[s..s + 80];
        let ph = rest.find("\"phase\":").map(|k| rest.as_bytes()[k + 8]).expect("phase after id");
        assert!((b'0'..=b'3').contains(&ph), "phase digit {}", ph as char);
        from = s;
    }
}

#[test]
fn new_event_kinds_appear_with_required_fields() {
    let mut all = String::new();
    for s in 0..14u64 {
        all.push_str(&logged(Brain::Skill(3), Brain::Skill(2), s));
        all.push_str(&logged(Brain::Skill(3), Brain::Skill(3), s + 100));
        all.push_str(&logged(Brain::Skill(2), Brain::Skill(3), s + 200));
    }
    let required: [(&str, &[&str]); 7] = [
        ("move", &["\"t\":", "\"who\":", "\"name\":", "\"reach\":", "\"speed\":", "\"arc\":", "\"delay\":"]),
        ("whiff", &["\"t\":", "\"who\":", "\"name\":"]),
        ("punish", &["\"t\":", "\"who\":", "\"victim\":", "\"name\":", "\"mult\":"]),
        ("guard_break", &["\"t\":", "\"who\":", "\"victim\":"]),
        ("feint", &["\"t\":", "\"who\":"]),
        ("counter", &["\"t\":", "\"who\":", "\"name\":"]),
        ("riposte_open", &["\"t\":", "\"who\":"]),
    ];
    for (kind, fields) in required {
        let evs = events_of(&all, kind);
        assert!(!evs.is_empty(), "no {} event in 42 skillbot fights", kind);
        for e in evs { for f in fields { assert!(e.contains(f), "{} event lacks {}: {}", kind, f, e); } }
    }
}
