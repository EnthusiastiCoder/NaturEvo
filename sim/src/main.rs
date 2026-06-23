use jungle_sim::report::{batch, run_json, stats_json};
use jungle_sim::sim::simulate;
use std::time::Instant;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn write(path: &str, content: &str) {
    if let Some(p) = std::path::Path::new(path).parent() {
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(p).expect("create dir");
        }
    }
    std::fs::write(path, content).expect("write file");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let t = Instant::now();
    match args.first().map(|s| s.as_str()) {
        Some("run") => {
            let seed: u64 = arg(&args, "--seed").and_then(|v| v.parse().ok()).unwrap_or(1);
            let out = arg(&args, "--out").unwrap_or_else(|| "viewer/data/run.js".into());
            let r = simulate(seed);
            write(&out, &run_json(&r));
            println!("seed {} won={} deaths={} level={} events={} -> {} ({:?})",
                seed, r.won, r.deaths, r.final_level, r.events.len(), out, t.elapsed());
        }
        Some("batch") => {
            let runs: u32 = arg(&args, "--runs").and_then(|v| v.parse().ok()).unwrap_or(2000);
            let out = arg(&args, "--out").unwrap_or_else(|| "viewer/data/stats.js".into());
            let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
            let s = batch(runs, threads);
            write(&out, &stats_json(&s));
            println!("runs {} win rate {:.1}% avg deaths {:.2} avg level {:.1} -> {} ({:?})",
                s.runs, 100.0 * s.wins as f64 / s.runs.max(1) as f64, s.avg_deaths, s.avg_final_level, out, t.elapsed());
        }
        Some("serve") => {
            let port: u16 = arg(&args, "--port").and_then(|v| v.parse().ok()).unwrap_or(8080);
            let root = arg(&args, "--root")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../viewer")));
            let l = jungle_sim::server::bind(port).unwrap_or_else(|e| {
                eprintln!("cannot bind 127.0.0.1:{}: {}", port, e);
                std::process::exit(1);
            });
            println!("Serving http://127.0.0.1:{}/", port);
            jungle_sim::server::serve_listener(l, root);
        }
        _ => {
            eprintln!("usage: jungle-sim run --seed N [--out path] | batch --runs N [--out path] | serve [--port N] [--root DIR]");
            std::process::exit(2);
        }
    }
}
