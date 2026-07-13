use jungle_sim::report::{batch, run_json, stats_json};
use jungle_sim::sim::{simulate, Event, HeroSnap, Run};

#[test]
fn run_json_shape_and_deterministic() {
    let a = run_json(&simulate(5));
    let b = run_json(&simulate(5));
    assert!(a.starts_with("window.RUN"));
    assert!(a.trim_end().ends_with(';'));
    assert_eq!(a.as_bytes(), b.as_bytes());
}

#[test]
fn json_escapes_quotes() {
    let run = Run {
        seed: 0,
        events: vec![Event {
            idx: 0,
            kind: "travel",
            text: "say \"hi\"".to_string(),
            hero: HeroSnap { level: 1, hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 },
            mon: None,
        }],
        won: false,
        deaths: 0,
        final_level: 1,
        first_win_event: None,
        end_level: 1,
        post_kills: 0,
        post_deaths: 0,
        max_stage: 0,
        capped: false,
    };
    let j = run_json(&run);
    assert!(j.contains("say \\\"hi\\\""), "{}", j);
}
