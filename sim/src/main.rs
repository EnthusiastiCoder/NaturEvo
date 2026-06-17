use jungle_sim::report::{batch, run_json, stats_json};
use jungle_sim::sim::simulate;
use std::time::Instant;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}
