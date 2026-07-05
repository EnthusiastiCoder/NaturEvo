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

pub fn run_body(run: &Run) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "{{\"seed\":{},\"won\":{},\"deaths\":{},\"final_level\":{},\"events\":[",
        run.seed, run.won, run.deaths, run.final_level
    ));
    for (i, e) in run.events.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let h = &e.hero;
        s.push_str(&format!(
            "{{\"idx\":{},\"kind\":\"{}\",\"text\":\"{}\",\"hero\":{{\"level\":{},\"hp\":{},\"max_hp\":{},\"xp\":{},\"gold\":{},\"potions\":{},\"zone\":{},\"weapon\":{},\"armor\":{}}},\"mon\":",
            e.idx, esc(e.kind), esc(&e.text), h.level, h.hp, h.max_hp, h.xp, h.gold, h.potions, h.zone, h.weapon, h.armor
        ));
        match &e.mon {
            Some(m) => s.push_str(&format!(
                "{{\"species\":{},\"level\":{},\"hp\":{},\"max_hp\":{}}}}}",
                m.species, m.level, m.hp, m.max_hp
            )),
            None => s.push_str("null}"),
        }
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
}

pub fn batch(runs: u32, threads: usize) -> BatchStats {
    batch_with(runs, threads, &Config::default())
}

pub fn batch_with(runs: u32, threads: usize, cfg: &Config) -> BatchStats {
    let threads = threads.max(1);
    // (won, deaths, final_level) per seed, indexed by seed
    let mut results = vec![(false, 0u32, 0u32); runs as usize];
    let chunk = ((runs as usize + threads - 1) / threads).max(1);
    std::thread::scope(|sc| {
        for (ci, part) in results.chunks_mut(chunk).enumerate() {
            sc.spawn(move || {
                for (j, slot) in part.iter_mut().enumerate() {
                    let r = simulate_with((ci * chunk + j) as u64, cfg);
                    *slot = (r.won, r.deaths, r.final_level);
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
