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
