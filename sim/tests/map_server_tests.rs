//! GET /api/map and map fields on /api/run (plan 2026-10-21_hero-map-walking, workstream 2).
mod common;
use common::parse;
use jungle_sim::map::{map_body, Map};
use jungle_sim::server::{route, Response};
use std::path::PathBuf;
