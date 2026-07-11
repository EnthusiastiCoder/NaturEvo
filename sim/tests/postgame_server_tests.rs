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
fn is_400(t: &str) {
    let r = get(t);
    assert_eq!(r.status, 400, "{} -> {}", t, text(&r).chars().take(120).collect::<String>());
    assert!(text(&r).contains("\"error\""), "{}", t);
}

#[test]
fn config_has_event_cap_and_max_events() {
    let b = text(&get("/api/config"));
    assert!(b.contains("\"event_cap\""), "{}", b);
    assert!(b.contains("\"postgame_level_cap\"") && b.contains("\"evolve_kills\""));
    assert!(b.contains("\"max_events\":1048576"), "{}", b);
}

#[test]
fn event_cap_above_ceiling_400() {
    is_400("/api/run?seed=1&event_cap=1048577&from=0&count=10");
    is_400("/api/batch?runs=1&event_cap=1048577");
}

#[test]
fn event_cap_2000_returns_2000_events() {
    let r = get("/api/run?seed=1&event_cap=2000");
    assert_eq!(r.status, 200);
    let b = text(&r);
    assert_eq!(b.matches("\"idx\":").count(), 2000);
    assert!(b.contains("\"total\":2000"));
}

#[test]
fn full_run_over_200k_without_window_400() {
    let r = get("/api/run?seed=1&event_cap=300000");
    assert_eq!(r.status, 400);
    let b = text(&r);
    assert!(b.contains("\"error\"") && b.contains("from") && b.contains("count"), "{}", b);
}
