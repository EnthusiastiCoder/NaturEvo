//! Third browser review: consistent end event with permadeath off, cumulative counters in run windows.
mod common;
use common::parse;
use jungle_sim::config::Config;
use jungle_sim::report::run_window_body;
use jungle_sim::sim::simulate_with;
