//! Post-game / event-cap / evolution tests (plan 2026-10-20_jungle-sim-postgame-evolution).
//! Written before the implementation; derived from the plan's API contract.
use jungle_sim::config::{knobs, Config, MAX_EVENT_CAP};
use jungle_sim::data::{stage_name, BOSS, EVOLUTIONS, MAX_STAGE, SPECIES, STAGE_MULT};
use jungle_sim::report::{batch_with, run_body, run_window_body, stats_body};
use jungle_sim::sim::{simulate_summary, simulate_with, Run};
use std::collections::HashMap;
use std::time::Instant;

/// Post-game scenarios assume the respawn loop, so the base config has permadeath off (Addendum 4).
fn base() -> Config {
    let mut c = Config::default();
    c.set("permadeath", "false").unwrap();
    c
}
fn cfg(k: &[(&str, &str)]) -> Config {
    let mut c = base();
    for (a, b) in k {
        c.set(a, b).unwrap();
    }
    c
}
fn stage_of(e: &jungle_sim::sim::Event) -> u8 {
    e.mon.as_ref().map(|m| m.stage).unwrap_or(0)
}
fn fw(r: &Run) -> usize {
    r.first_win_event.expect("expected a win") as usize
}
/// value of a numeric key like `"total":123`
fn num_after(body: &str, key: &str) -> Option<u64> {
    let p = format!("\"{}\":", key);
    let i = body.find(&p)? + p.len();
    let d: String = body[i..].chars().take_while(|c| c.is_ascii_digit()).collect();
    d.parse().ok()
}
fn array_len(body: &str, key: &str) -> usize {
    let p = format!("\"{}\":[", key);
    let i = body.find(&p).unwrap_or_else(|| panic!("missing {}", key)) + p.len();
    let end = body[i..].find(']').unwrap();
    let inner = &body[i..i + end];
    if inner.trim().is_empty() { 0 } else { inner.split(',').count() }
}

// ---------------- config ----------------
#[test]
fn cfg_defaults() {
    let c = Config::default();
    assert_eq!((c.event_cap, c.postgame_level_cap, c.evolve_kills), (65536, 80, 300));
    assert_eq!(MAX_EVENT_CAP, 1_048_576);
}

#[test]
fn cfg_sixteen_knobs_with_ranges() {
    assert_eq!(knobs().len(), 24);
    let find = |k: &str| knobs().iter().find(|x| x.key == k).unwrap_or_else(|| panic!("no knob {}", k));
    for (k, lo, hi, d) in [
        ("event_cap", 100.0, 1_048_576.0, 65536.0),
        ("postgame_level_cap", 40.0, 200.0, 80.0),
        ("evolve_kills", 10.0, 100000.0, 300.0),
    ] {
        let n = find(k);
        assert_eq!((n.min, n.max, n.default), (lo, hi, d), "{}", k);
    }
}

#[test]
fn cfg_set_accepts_bounds() {
    let mut c = Config::default();
    for (k, v) in [
        ("event_cap", "100"), ("event_cap", "1048576"),
        ("postgame_level_cap", "40"), ("postgame_level_cap", "200"),
        ("evolve_kills", "10"), ("evolve_kills", "100000"),
    ] {
        c.set(k, v).unwrap_or_else(|e| panic!("{}={} rejected: {}", k, v, e));
    }
    assert_eq!((c.event_cap, c.postgame_level_cap, c.evolve_kills), (1_048_576, 200, 100000));
    assert!(c.validate().is_ok());
}

#[test]
fn cfg_set_rejects_out_of_range_and_garbage() {
    let mut c = Config::default();
    for (k, v) in [
        ("event_cap", "99"), ("event_cap", "1048577"), ("event_cap", "abc"), ("event_cap", "NaN"),
        ("postgame_level_cap", "39"), ("postgame_level_cap", "201"), ("postgame_level_cap", "abc"), ("postgame_level_cap", "NaN"),
        ("evolve_kills", "9"), ("evolve_kills", "100001"), ("evolve_kills", "abc"), ("evolve_kills", "NaN"),
    ] {
        let e = c.set(k, v).unwrap_err();
        assert!(e.contains(k), "{}={}: {}", k, v, e);
    }
    assert_eq!(c, Config::default());
}

// ---------------- prefix / baselines ----------------
#[test]
fn prefix_first_win_events_and_level() {
    for (seed, f) in [(7u64, 15436u32), (0, 15619), (5, 15844)] {
        let r = simulate_with(seed, &base());
        assert!(r.won, "seed {}", seed);
        assert_eq!(r.first_win_event, Some(f), "seed {}", seed);
        assert_eq!(r.final_level, 40, "seed {}", seed);
        assert_eq!(r.events[f as usize].kind, "win", "seed {}", seed);
        assert!(r.events[..=f as usize].iter().all(|e| stage_of(e) == 0), "seed {}", seed);
    }
}

#[test]
fn prefix_small_cap_is_prefix_of_default() {
    for seed in [7u64, 3, 1] {
        let full = simulate_with(seed, &base());
        let short = simulate_with(seed, &cfg(&[("event_cap", "5000")]));
        assert_eq!(short.events.len(), 5000);
        for (a, b) in short.events.iter().zip(full.events.iter()) {
            assert_eq!((a.idx, a.kind, &a.text), (b.idx, b.kind, &b.text), "seed {}", seed);
            assert_eq!((a.hero.level, a.hero.hp, a.hero.gold, a.hero.zone), (b.hero.level, b.hero.hp, b.hero.gold, b.hero.zone));
        }
    }
}

#[test]
fn prefix_across_win_boundary() {
    let full = simulate_with(7, &base());
    let mid = simulate_with(7, &cfg(&[("event_cap", "20000")]));
    assert_eq!(mid.first_win_event, Some(15436));
    assert_eq!(mid.events.len(), 20000);
    for (a, b) in mid.events.iter().zip(full.events.iter()) {
        assert_eq!((a.kind, &a.text), (b.kind, &b.text));
    }
}

#[test]
fn baseline_win_counts_unchanged() {
    assert_eq!(batch_with(200, 4, &base()).wins, 81);
    assert_eq!(batch_with(2000, 4, &base()).wins, 905);
}

// ---------------- cap ----------------
#[test]
fn cap_default_winning_run_is_capped_at_65536() {
    let r = simulate_with(7, &base());
    assert_eq!(r.events.len(), 65536);
    assert!(r.capped);
}

#[test]
fn cap_2000_exact_and_consecutive() {
    let r = simulate_with(7, &cfg(&[("event_cap", "2000")]));
    assert_eq!(r.events.len(), 2000);
    assert!(r.capped);
    assert!(!r.won, "win happens at 15436, after the cap");
    for (i, e) in r.events.iter().enumerate() {
        assert_eq!(e.idx as usize, i);
    }
}

#[test]
fn cap_len_never_exceeds_cap() {
    for cap in [100u32, 777, 5000] {
        for seed in 0..10u64 {
            let r = simulate_with(seed, &cfg(&[("event_cap", &cap.to_string())]));
            assert!(r.events.len() <= cap as usize, "cap {} seed {}", cap, seed);
        }
    }
}

#[test]
fn cap_losing_run_ends_without_postgame() {
    let seed = (0..200u64).find(|s| !simulate_with(*s, &base()).won).expect("a losing seed in 0..200");
    let r = simulate_with(seed, &base());
    assert!(r.events.len() < 65536);
    assert!(!r.capped);
    assert_eq!(r.first_win_event, None);
    assert_eq!(r.post_kills, 0);
    assert_eq!(r.post_deaths, 0);
    assert_eq!(r.max_stage, 0);
    assert!(r.events.iter().all(|e| e.kind != "win" && e.kind != "evolve"));
}

// ---------------- post-game ----------------
#[test]
fn post_exactly_one_win_event_and_continues() {
    let r = simulate_with(7, &base());
    let wins: Vec<usize> = r.events.iter().enumerate().filter(|(_, e)| e.kind == "win").map(|(i, _)| i).collect();
    assert_eq!(wins, vec![15436]);
    assert!(r.events.len() > 10625 + 1000);
}

#[test]
fn post_never_targets_boss_again() {
    let r = simulate_with(7, &base());
    let f = fw(&r);
    assert!(r.events[f + 1..].iter().all(|e| e.kind != "boss"));
    assert!(r.events[f + 1..].iter().all(|e| e.hero.zone != 8), "zone 8 is the boss zone");
}

#[test]
fn post_levels_past_40_up_to_postgame_cap() {
    let r = simulate_with(7, &base());
    assert!(r.end_level > 40, "end_level {}", r.end_level);
    assert!(r.end_level <= 80);
    assert!(r.events.iter().all(|e| e.hero.level <= 80));
    let r40 = simulate_with(7, &cfg(&[("postgame_level_cap", "40")]));
    assert_eq!(r40.end_level, 40);
    let r200 = simulate_with(7, &cfg(&[("postgame_level_cap", "200")]));
    assert!(r200.end_level >= r.end_level && r200.end_level <= 200);
}

#[test]
fn post_hp_invariants() {
    for seed in [7u64, 3] {
        for e in &simulate_with(seed, &base()).events {
            assert!(e.hero.hp >= 0 && e.hero.hp <= e.hero.max_hp, "seed {} idx {}", seed, e.idx);
            if let Some(m) = &e.mon {
                assert!(m.hp <= m.max_hp, "seed {} idx {}", seed, e.idx);
            }
        }
    }
}

#[test]
fn post_deaths_field_is_prewin_count() {
    for seed in [7u64, 0, 5] {
        let r = simulate_with(seed, &base());
        let f = fw(&r);
        let pre = r.events[..=f].iter().filter(|e| e.kind == "death").count() as u32;
        assert_eq!(r.deaths, pre, "seed {}", seed);
        let post = r.events[f + 1..].iter().filter(|e| e.kind == "death").count() as u32;
        assert_eq!(r.post_deaths, post, "seed {}", seed);
    }
}

#[test]
fn post_kills_counts_victories_after_win() {
    let r = simulate_with(7, &base());
    let f = fw(&r);
    let v = r.events[f + 1..].iter().filter(|e| e.kind == "victory_fight").count() as u32;
    assert_eq!(r.post_kills, v);
    assert!(r.post_kills > 0);
}

// ---------------- evolution ----------------
#[test]
fn evo_tables() {
    assert_eq!(MAX_STAGE, 4);
    assert_eq!(STAGE_MULT[0], 1.0);
    assert!(STAGE_MULT.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(EVOLUTIONS.len(), 16);
    for (i, s) in SPECIES.iter().enumerate() {
        assert_eq!(EVOLUTIONS[i][0], s.name);
        assert_eq!(stage_name(i, 0), s.name);
        for st in 0..5 {
            assert!(!EVOLUTIONS[i][st].is_empty());
            assert_eq!(stage_name(i, st), EVOLUTIONS[i][st]);
        }
        if i != BOSS {
            let mut v = EVOLUTIONS[i].to_vec();
            v.sort();
            v.dedup();
            assert_eq!(v.len(), 5, "line {} names not distinct", i);
        }
    }
    assert!(EVOLUTIONS[BOSS].iter().all(|n| *n == "Jungle King"));
}

#[test]
fn evo_fast_kills_reach_stage_4_with_four_events() {
    let r = simulate_with(7, &cfg(&[("evolve_kills", "10")]));
    assert_eq!(r.max_stage, 4);
    let n = r.events.iter().filter(|e| e.kind == "evolve").count();
    assert_eq!(n, 4);
    assert!(r.events.iter().filter(|e| e.kind == "evolve").all(|e| e.mon.is_none()));
    let f = fw(&r);
    let mut last = 0u8;
    for e in r.events[f + 1..].iter().filter(|e| e.kind == "encounter") {
        let s = stage_of(e);
        assert!(s >= last, "stage went backwards at {}", e.idx);
        last = s;
    }
    assert_eq!(last, 4);
    assert!(r.events.iter().all(|e| stage_of(e) <= 4));
}

#[test]
fn evo_slow_kills_never_evolve() {
    let r = simulate_with(7, &cfg(&[("evolve_kills", "100000")]));
    assert_eq!(r.max_stage, 0);
    assert!(r.events.iter().all(|e| e.kind != "evolve" && stage_of(e) == 0));
}

#[test]
fn evo_boss_never_evolved_and_prewin_stage_zero() {
    let r = simulate_with(7, &cfg(&[("evolve_kills", "10")]));
    let f = fw(&r);
    assert!(r.events[..=f].iter().all(|e| stage_of(e) == 0));
    assert!(r.events.iter().filter_map(|e| e.mon.as_ref()).filter(|m| m.species == BOSS).all(|m| m.stage == 0));
}

#[test]
fn evo_encounter_text_uses_stage_name() {
    let r = simulate_with(7, &cfg(&[("evolve_kills", "10")]));
    let mut n = 0;
    for e in r.events.iter().filter(|e| e.kind == "encounter") {
        if let Some(m) = &e.mon {
            if m.stage > 0 {
                n += 1;
                let nm = stage_name(m.species, m.stage as usize);
                assert!(e.text.contains(nm), "text {:?} lacks {:?}", e.text, nm);
            }
        }
    }
    assert!(n > 50, "expected many evolved encounters, got {}", n);
}

#[test]
fn evo_stage_monsters_are_stronger_at_same_level() {
    let mut base: HashMap<(usize, u32), i32> = HashMap::new();
    let mut evolved = Vec::new();
    for seed in [7u64, 3, 1] {
        for e in simulate_with(seed, &cfg(&[("evolve_kills", "10")])).events {
            if let Some(m) = e.mon {
                if m.species == BOSS {
                    continue;
                }
                if m.stage == 0 {
                    base.insert((m.species, m.level), m.max_hp);
                } else {
                    evolved.push(m);
                }
            }
        }
    }
    let mut compared = 0;
    for m in evolved {
        if let Some(b) = base.get(&(m.species, m.level)) {
            compared += 1;
            assert!(m.max_hp > *b, "species {} lvl {} stage {}: {} <= {}", m.species, m.level, m.stage, m.max_hp, b);
        }
    }
    assert!(compared > 0, "no stage>0 / stage-0 pair with equal species and level");
}

// ---------------- determinism / summary ----------------
#[test]
fn det_same_seed_same_cfg() {
    let c = cfg(&[("evolve_kills", "10")]);
    let a = simulate_with(7, &c);
    let b = simulate_with(7, &c);
    assert_eq!(a.events.len(), b.events.len());
    for (x, y) in a.events.iter().zip(b.events.iter()) {
        assert_eq!((x.kind, &x.text, stage_of(x)), (y.kind, &y.text, stage_of(y)));
    }
    assert_eq!(a.summary(), b.summary());
}

#[test]
fn det_evolve_kills_changes_run() {
    let a = simulate_with(7, &cfg(&[("evolve_kills", "10")]));
    let b = simulate_with(7, &cfg(&[("evolve_kills", "100")]));
    let ta: Vec<&str> = a.events.iter().map(|e| e.text.as_str()).collect();
    let tb: Vec<&str> = b.events.iter().map(|e| e.text.as_str()).collect();
    assert_ne!(ta, tb);
}

#[test]
fn summary_matches_lean_summary() {
    let cfgs = [
        Config::default(),
        cfg(&[("event_cap", "3000")]),
        cfg(&[("event_cap", "100")]),
        cfg(&[("evolve_kills", "10"), ("boss_hp_mult", "2.5")]),
        cfg(&[("postgame_level_cap", "40"), ("hero_hp_per_level", "20")]),
    ];
    for c in &cfgs {
        for seed in 0..20u64 {
            let full = simulate_with(seed, c);
            let s = full.summary();
            assert_eq!(simulate_summary(seed, c), s, "seed {}", seed);
            assert_eq!(s.events as usize, full.events.len());
            assert_eq!(s.won, full.won);
            assert_eq!(s.first_win_event, full.first_win_event);
        }
    }
}

// ---------------- stats / report ----------------
#[test]
fn stats_stage_hist_sums_to_wins() {
    for c in [Config::default(), cfg(&[("evolve_kills", "10")])] {
        let s = batch_with(60, 2, &c);
        assert_eq!(s.stage_hist.len(), 5);
        assert_eq!(s.stage_hist.iter().sum::<u32>(), s.wins);
    }
    let s = batch_with(30, 2, &cfg(&[("evolve_kills", "10")]));
    assert!(s.stage_hist[4] > 0);
}

#[test]
fn stats_default_averages() {
    let s = batch(100, 2);
    assert!(s.avg_events > 10000.0, "avg_events {}", s.avg_events);
    assert!(s.avg_first_win_event > 5000.0 && s.avg_first_win_event < 65536.0, "{}", s.avg_first_win_event);
    assert!(s.avg_post_kills > 0.0);
    assert!(s.avg_post_deaths >= 0.0);
}

#[test]
fn stats_thread_independent_nondefault() {
    let c = cfg(&[("evolve_kills", "20"), ("event_cap", "20000"), ("boss_hp_mult", "1.5")]);
    let a = batch_with(40, 1, &c);
    let b = batch_with(40, 4, &c);
    assert_eq!(a.wins, b.wins);
    assert_eq!(a.avg_events, b.avg_events);
    assert_eq!(a.avg_first_win_event, b.avg_first_win_event);
    assert_eq!(a.avg_post_kills, b.avg_post_kills);
    assert_eq!(a.avg_post_deaths, b.avg_post_deaths);
    assert_eq!(a.stage_hist, b.stage_hist);
    assert_eq!(a.level_hist, b.level_hist);
}

#[test]
fn stats_body_has_new_keys() {
    let b = stats_body(&batch(20, 2));
    for k in ["avg_events", "avg_first_win_event", "avg_post_kills", "avg_post_deaths", "stage_hist", "wins", "level_hist"] {
        assert!(b.contains(&format!("\"{}\":", k)), "missing {} in {}", k, b);
    }
    assert_eq!(array_len(&b, "stage_hist"), 5);
}

#[test]
fn report_run_body_new_top_level_keys() {
    let r = simulate(7);
    let b = run_body(&r);
    for k in ["first_win_event", "end_level", "post_kills", "post_deaths", "max_stage", "capped", "total"] {
        assert!(b.contains(&format!("\"{}\":", k)), "missing {}", k);
    }
    assert_eq!(num_after(&b, "first_win_event"), Some(14161));
    assert_eq!(num_after(&b, "total"), Some(65536));
    assert!(b.contains("\"capped\":true"));
    let lose = (0..200u64).find(|s| !simulate(*s).won).unwrap();
    let lb = run_body(&simulate(lose));
    assert!(lb.contains("\"first_win_event\":null"));
    assert!(lb.contains("\"capped\":false"));
}

#[test]
fn report_stage_key_only_when_positive() {
    let pre = run_body(&simulate_with(7, &cfg(&[("event_cap", "5000")])));
    assert!(!pre.contains("\"stage\""), "pre-win events must not carry stage");
    let post = run_body(&simulate_with(7, &cfg(&[("evolve_kills", "10"), ("event_cap", "20000")])));
    assert!(post.contains("\"stage\":1") && post.contains("\"stage\":4"));
    assert!(!post.contains("\"stage\":0"));
}

#[test]
fn report_window_slice_and_clamp() {
    let r = simulate(7);
    let w = run_window_body(&r, 1000, 50);
    assert_eq!(num_after(&w, "from"), Some(1000));
    assert_eq!(num_after(&w, "total"), Some(65536));
    assert_eq!(w.matches("\"idx\":").count(), 50);
    assert!(w.contains("\"idx\":1000,") && w.contains("\"idx\":1049,") && !w.contains("\"idx\":1050,"));
    for k in ["first_win_event", "end_level", "post_kills", "capped", "level_series"] {
        assert!(w.contains(&format!("\"{}\":", k)), "missing {}", k);
    }
    let tail = run_window_body(&r, 65530, 100);
    assert_eq!(tail.matches("\"idx\":").count(), 6);
    let past = run_window_body(&r, 999_999, 10);
    assert_eq!(past.matches("\"idx\":").count(), 0);
    assert!(past.starts_with('{') && past.trim_end().ends_with('}'));
}

#[test]
fn report_level_series_bounded_and_sampled() {
    let r = simulate(7);
    let n = array_len(&run_window_body(&r, 0, 10), "level_series");
    assert!(n >= 1000 && n <= 2048, "level_series len {}", n);
    let small = simulate_with(7, &cfg(&[("event_cap", "300")]));
    let n2 = array_len(&run_window_body(&small, 0, 10), "level_series");
    assert!(n2 >= 1 && n2 <= 300, "{}", n2);
}

#[test]
fn perf_batch_2000_under_15s() {
    let t = Instant::now();
    let s = batch(2000, 4);
    let el = t.elapsed().as_secs_f64();
    assert_eq!(s.runs, 2000);
    assert!(el < 15.0, "batch(2000) took {:.1}s", el);
}

#[test]
fn losing_seed_3_has_no_postgame() {
    // seed 3 loses to the boss on the walking build: 14776 events, level 40, never wins
    let r = simulate(3);
    assert!(!r.won);
    assert_eq!(r.first_win_event, None);
    assert_eq!(r.events.len(), 14776);
    assert_eq!(r.final_level, 40);
    assert!(!r.capped);
    assert_eq!(r.post_kills, 0);
    assert_eq!(r.max_stage, 0);
    assert!(r.events.len() <= 65536);
    assert!(r.events.iter().all(|e| e.kind != "win" && e.kind != "evolve"));
    let small = simulate_with(3, &cfg(&[("event_cap", "5000")]));
    assert_eq!(small.events.len(), 5000);
    assert!(small.capped);
    assert_eq!(small.first_win_event, None);
}
