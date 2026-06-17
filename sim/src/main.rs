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
