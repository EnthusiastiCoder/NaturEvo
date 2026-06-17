use crate::sim::{simulate, Run};

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
    let mut s = String::new();
    s.push_str(&format!(
        "window.RUN = {{\"seed\":{},\"won\":{},\"deaths\":{},\"final_level\":{},\"events\":[",
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
    s.push_str("]};\n");
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
