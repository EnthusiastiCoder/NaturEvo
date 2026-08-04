//! GET /api/map and map fields on /api/run (plan 2026-10-21_hero-map-walking, workstream 2).
mod common;
use common::parse;
use jungle_sim::map::{map_body, Map};
use jungle_sim::server::{route, Response};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../viewer"))
}
fn get(t: &str) -> Response {
    route(&root(), "GET", t, Some("127.0.0.1:8080"))
}
fn text(r: &Response) -> String {
    String::from_utf8(r.body.clone()).unwrap()
}
