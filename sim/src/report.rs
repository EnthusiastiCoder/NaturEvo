use crate::config::Config;
use crate::sim::{simulate_summary, Run, Summary};

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o
}

pub fn run_json(run: &Run) -> String {
    format!("window.RUN = {};\n", run_body(run))
}

fn head_fields(run: &Run) -> String {
    let fw = match run.first_win_event {
        Some(v) => v.to_string(),
        None => "null".to_string(),
    };
    format!(
        "\"seed\":{},\"won\":{},\"deaths\":{},\"final_level\":{},\"first_win_event\":{},\"end_level\":{},\"post_kills\":{},\"post_deaths\":{},\"max_stage\":{},\"capped\":{},\"total\":{}",
        run.seed, run.won, run.deaths, run.final_level, fw, run.end_level, run.post_kills,
        run.post_deaths, run.max_stage, run.capped, run.events.len()
    )
}

fn push_event(s: &mut String, e: &crate::sim::Event) {
    let h = &e.hero;
    s.push_str(&format!(
        "{{\"idx\":{},\"kind\":\"{}\",\"text\":\"{}\",\"hero\":{{\"level\":{},\"hp\":{},\"max_hp\":{},\"xp\":{},\"gold\":{},\"potions\":{},\"zone\":{},\"weapon\":{},\"armor\":{}}},\"mon\":",
        e.idx, esc(e.kind), esc(&e.text), h.level, h.hp, h.max_hp, h.xp, h.gold, h.potions, h.zone, h.weapon, h.armor
    ));
    match &e.mon {
        Some(m) => {
            s.push_str(&format!(
                "{{\"species\":{},\"level\":{},\"hp\":{},\"max_hp\":{}",
                m.species, m.level, m.hp, m.max_hp
            ));
            if m.stage > 0 {
                s.push_str(&format!(",\"stage\":{}", m.stage));
            }
            s.push_str("}}");
        }
        None => s.push_str("null}"),
    }
}

pub fn run_body(run: &Run) -> String {
    let mut s = String::new();
    s.push('{');
    s.push_str(&head_fields(run));
    s.push_str(",\"events\":[");
    for (i, e) in run.events.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        push_event(&mut s, e);
    }
    s.push_str("]}");
    s
}

pub fn run_window_body(run: &Run, from: usize, count: usize) -> String {
    let n = run.events.len();
    let from = from.min(n);
    let end = from.saturating_add(count).min(n);
    let mut s = String::new();
    s.push('{');
    s.push_str(&head_fields(run));
    s.push_str(&format!(",\"from\":{},\"events\":[", from));
    for (i, e) in run.events[from..end].iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        push_event(&mut s, e);
    }
    s.push_str("],\"level_series\":[");
    let pts = n.min(2048);
    for k in 0..pts {
        if k > 0 {
            s.push(',');
        }
        let i = k * n / pts;
        s.push_str(&run.events[i].hero.level.to_string());
    }
    s.push_str("]}");
    s
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchStats {
    pub runs: u32,
    pub wins: u32,
    pub avg_deaths: f64,
    pub avg_final_level: f64,
    pub level_hist: Vec<u32>,
    pub avg_events: f64,
    pub avg_first_win_event: f64,
    pub avg_post_kills: f64,
    pub avg_post_deaths: f64,
    pub stage_hist: Vec<u32>,
}

pub fn batch(runs: u32, threads: usize) -> BatchStats {
    batch_with(runs, threads, &Config::default())
}

pub fn batch_with(runs: u32, threads: usize, cfg: &Config) -> BatchStats {
    let threads = threads.max(1);
    let mut results: Vec<Summary> = Vec::with_capacity(runs as usize);
    results.resize(
        runs as usize,
        Summary {
            won: false,
            deaths: 0,
            final_level: 0,
            first_win_event: None,
            events: 0,
            end_level: 0,
            post_kills: 0,
            post_deaths: 0,
            max_stage: 0,
            capped: false,
        },
    );
    let chunk = ((runs as usize + threads - 1) / threads).max(1);
    std::thread::scope(|sc| {
        for (ci, part) in results.chunks_mut(chunk).enumerate() {
            sc.spawn(move || {
                for (j, slot) in part.iter_mut().enumerate() {
                    *slot = simulate_summary((ci * chunk + j) as u64, cfg);
                }
            });
        }
    });
    let mut wins = 0u32;
    let mut deaths = 0u64;
    let mut lv = 0u64;
    let mut hist = vec![0u32; 41];
    for (w, d, l) in &results {
        if *w {
            wins += 1;
        }
        deaths += *d as u64;
        lv += *l as u64;
        hist[(*l as usize).min(40)] += 1;
    }
    let n = runs.max(1) as f64;
    BatchStats {
        runs,
        wins,
        avg_deaths: deaths as f64 / n,
        avg_final_level: lv as f64 / n,
        level_hist: hist,
    }
}

pub fn stats_json(s: &BatchStats) -> String {
    format!("window.STATS = {};\n", stats_body(s))
}

pub fn stats_body(s: &BatchStats) -> String {
    let hist: Vec<String> = s.level_hist.iter().map(|v| v.to_string()).collect();
    format!(
        "{{\"runs\":{},\"wins\":{},\"avg_deaths\":{},\"avg_final_level\":{},\"level_hist\":[{}]}}",
        s.runs, s.wins, s.avg_deaths, s.avg_final_level, hist.join(",")
    )
}
