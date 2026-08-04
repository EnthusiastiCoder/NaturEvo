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

#[test]
fn map_route_shape_and_defaults() {
    let r = get("/api/map?seed=1");
    assert_eq!(r.status, 200);
    let j = parse(&text(&r)).unwrap();
    assert_eq!((j.u("w"), j.u("h")), (64, 64), "default map_size is 64");
    let tiles = j.get("tiles").unwrap().arr();
    let zones = j.get("zones").unwrap().arr();
    assert_eq!(tiles.len(), 64);
    assert_eq!(zones.len(), 64);
    for (t, z) in tiles.iter().zip(zones) {
        assert_eq!(t.str().chars().count(), 64);
        assert_eq!(z.str().chars().count(), 64);
        assert!(t.str().chars().all(|c| ".,#C".contains(c)), "{}", t.str());
        assert!(z.str().chars().all(|c| ('0'..='8').contains(&c)));
    }
    assert_eq!(tiles.iter().map(|t| t.str().matches('C').count()).sum::<usize>(), 1, "exactly one camp tile");
    let anchors = j.get("anchors").unwrap().arr();
    assert_eq!(anchors.len(), 9);
    for a in anchors {
        let (x, y) = a.xy();
        assert!(x < 64 && y < 64);
    }
    assert_eq!(text(&get("/api/map")), text(&r), "default seed is 1");
}
