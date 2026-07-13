use jungle_sim::config::Config;
use jungle_sim::report::{batch, batch_with, run_body, run_json, stats_body, stats_json};
use jungle_sim::sim::simulate;

#[test]
fn body_wrappers_equal_json() {
    let r = simulate(3);
    assert_eq!(run_json(&r), format!("window.RUN = {};\n", run_body(&r)));
    let s = batch(20, 2);
    assert_eq!(stats_json(&s), format!("window.STATS = {};\n", stats_body(&s)));
}

#[test]
fn batch_with_default_equals_batch() {
    let a = batch(60, 2);
    let b = batch_with(60, 2, &Config::default());
    assert_eq!(a.runs, b.runs);
    assert_eq!(a.wins, b.wins);
    assert_eq!(a.avg_deaths, b.avg_deaths);
    assert_eq!(a.avg_final_level, b.avg_final_level);
    assert_eq!(a.level_hist, b.level_hist);
}

#[test]
fn batch_with_thread_independent_nondefault() {
    let mut c = Config::default();
    c.boss_hp_mult = 2.5;
    c.hero_hp_per_level = 20;
    let a = batch_with(50, 1, &c);
    let b = batch_with(50, 4, &c);
    assert_eq!(a.wins, b.wins);
    assert_eq!(a.avg_deaths, b.avg_deaths);
    assert_eq!(a.avg_final_level, b.avg_final_level);
    assert_eq!(a.level_hist, b.level_hist);
}

#[test]
fn run_body_is_plain_json_object() {
    let b = run_body(&simulate(2));
    assert!(b.starts_with('{'));
    assert!(!b.contains("window"));
    let t = b.trim_end();
    assert!(t.ends_with('}'));
    let (mut depth, mut in_str, mut esc) = (0i64, false, false);
    for ch in t.chars() {
        if in_str {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                assert!(depth >= 0);
            }
            _ => {}
        }
    }
    assert_eq!(depth, 0);
    assert!(!in_str);
}
