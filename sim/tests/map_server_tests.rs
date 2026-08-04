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

#[test]
fn map_route_matches_generate() {
    let r = get("/api/map?seed=5&map_size=48");
    assert_eq!(r.status, 200, "{}", text(&r));
    assert!(r.content_type.starts_with("application/json"));
    assert_eq!(text(&r), map_body(&Map::generate(5, 48)));
}
