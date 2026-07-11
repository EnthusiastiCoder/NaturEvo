//! Post-game / event-cap / evolution tests (plan 2026-10-20_jungle-sim-postgame-evolution).
//! Written before the implementation; derived from the plan's API contract.
use jungle_sim::config::{knobs, Config, MAX_EVENT_CAP};
use jungle_sim::data::{stage_name, BOSS, EVOLUTIONS, MAX_STAGE, SPECIES, STAGE_MULT};
use jungle_sim::report::{batch, batch_with, run_body, run_window_body, stats_body};
use jungle_sim::sim::{simulate, simulate_summary, simulate_with, Run};
use std::collections::HashMap;
use std::time::Instant;
