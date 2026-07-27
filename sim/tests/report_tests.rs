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
        steps: 0,
        map_size: 64,
        events: vec![Event {
            idx: 0,
            kind: "travel",
            text: "say \"hi\"".to_string(),
            hero: HeroSnap { level: 1, hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0, x: 0, y: 0 },
            mon: None,
            npcs: vec![],
            path: vec![],
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

#[test]
fn batch_thread_independent() {
    let a = batch(50, 1);
    let b = batch(50, 4);
    assert_eq!(a.runs, b.runs);
    assert_eq!(a.wins, b.wins);
    assert_eq!(a.avg_deaths, b.avg_deaths);
    assert_eq!(a.avg_final_level, b.avg_final_level);
    assert_eq!(a.level_hist, b.level_hist);
}

#[test]
fn level_hist_sums_to_runs() {
    let s = batch(50, 2);
    assert_eq!(s.runs, 50);
    assert_eq!(s.level_hist.iter().sum::<u32>(), 50);
}

#[test]
fn stats_json_has_wins() {
    let j = stats_json(&batch(10, 1));
    assert!(j.contains("wins"));
}

#[test]
fn win_rate_sane() {
    let s = batch(500, 4);
    let r = s.wins as f64 / s.runs as f64;
    assert!((0.30..=0.90).contains(&r), "win rate {}", r);
}
