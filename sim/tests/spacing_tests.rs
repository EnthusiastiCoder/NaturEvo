//! NPC and hero spacing, roaming limits and encounter pacing (plan Addendum 4 part 2). Written before any implementation.
//! Homes are read as each NPC's position in the first event (events[0].npcs); hero start is events[0].hero.
//! Positions over time are rebuilt from the event npcs list plus the per-step nsteps moves.
//! Scaled minimum = max(6, base * size / 64). Zone anchors pairwise >= scaled 14.
mod common;
use common::{parse, J};
use jungle_sim::config::Config;
use jungle_sim::map::Map;
use jungle_sim::report::run_body;
use jungle_sim::sim::simulate_with;
use std::collections::HashMap;
