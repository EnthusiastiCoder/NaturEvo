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

#[test]
fn run_with_knob_equals_simulate_with() {
    let mut c = Config::default();
    c.hero_hp_per_level = 30;
    let r = get("/api/run?seed=5&hero_hp_per_level=30");
    assert_eq!(r.status, 200);
    assert_eq!(text(&r), run_body(&simulate_with(5, &c)));
}

#[test]
fn batch_50_matches_stats_body() {
    let r = get("/api/batch?runs=50");
    assert_eq!(r.status, 200);
    assert_eq!(text(&r), stats_body(&batch(50, 2)));
}

#[test]
fn bad_inputs_400() {
    for t in [
        "/api/run?seed=abc",
        "/api/run?seed=-1",
        "/api/run?bogus=1",
        "/api/run?boss_attempts=0",
        "/api/run?xp_mult=NaN",
        "/api/batch?runs=0",
        "/api/batch?runs=20001",
        "/api/batch?runs=x",
    ] {
        let r = get(t);
        assert_eq!(r.status, 400, "{}", t);
        assert!(text(&r).contains("\"error\""), "{}", t);
    }
}

#[test]
fn post_is_405() {
    assert_eq!(route(&root(), "POST", "/api/run", Some("127.0.0.1")).status, 405);
}

#[test]
fn bad_host_is_403() {
    assert_eq!(route(&root(), "GET", "/api/config", Some("evil.com")).status, 403);
    assert_eq!(route(&root(), "GET", "/", Some("evil.com:8080")).status, 403);
    assert_eq!(route(&root(), "GET", "/api/config", Some("localhost:9")).status, 200);
    assert_eq!(route(&root(), "GET", "/api/config", Some("127.0.0.1")).status, 200);
}

#[test]
fn traversal_is_404() {
    for t in ["/..%2f", "/../Cargo.toml", "/data/../x", "/data\\run.js", "/other.txt"] {
        assert_eq!(get(t).status, 404, "{}", t);
    }
}

#[test]
fn static_files() {
    let r = get("/");
    assert_eq!(r.status, 200);
    assert!(r.content_type.starts_with("text/html"));
    assert_eq!(r.body, std::fs::read(root().join("index.html")).unwrap());
    let j = get("/data/run.js");
    assert_eq!(j.status, 200);
    assert!(j.content_type.contains("javascript"));
}

fn start() -> u16 {
    let l = bind(0).unwrap();
    let addr = l.local_addr().unwrap();
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    let r = root();
    std::thread::spawn(move || serve_listener(l, r));
    addr.port()
}
