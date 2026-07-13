use jungle_sim::config::{knobs, Config};
use jungle_sim::report::{batch, run_body, stats_body};
use jungle_sim::server::{bind, route, serve_listener, Response};
use jungle_sim::sim::{simulate, simulate_with};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
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
fn config_lists_all_knob_keys() {
    let r = get("/api/config");
    assert_eq!(r.status, 200);
    assert!(r.content_type.starts_with("application/json"));
    let b = text(&r);
    for k in knobs() {
        assert!(b.contains(&format!("\"{}\"", k.key)), "{}", k.key);
    }
    assert!(b.contains("\"max_runs\":20000"), "{}", b);
}

#[test]
fn run_seed_1_matches_run_body() {
    let r = get("/api/run?seed=1");
    assert_eq!(r.status, 200);
    assert!(r.content_type.starts_with("application/json"));
    assert_eq!(text(&r), run_body(&simulate(1)));
}

#[test]
fn run_default_seed_is_1() {
    assert_eq!(text(&get("/api/run")), run_body(&simulate(1)));
}
