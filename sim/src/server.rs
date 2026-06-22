use crate::config::{knobs, Config};
use crate::report::{batch_with, run_body, stats_body};
use crate::sim::simulate_with;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_RUNS: u32 = 20000;
const MAX_REQ: usize = 8 * 1024;

pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

const JSON: &str = "application/json; charset=utf-8";

fn json_resp(status: u16, body: String) -> Response {
    Response { status, content_type: JSON, body: body.into_bytes() }
}

fn err(status: u16, msg: &str) -> Response {
    let esc: String = msg.chars().filter(|c| *c != '"' && *c != '\\' && (*c as u32) >= 0x20).collect();
    json_resp(status, format!("{{\"error\":\"{}\"}}", esc))
}

fn host_ok(host: Option<&str>) -> bool {
    let h = match host {
        Some(h) => h.trim(),
        None => return false,
    };
    let (name, port) = match h.split_once(':') {
        Some((n, p)) => (n, Some(p)),
        None => (h, None),
    };
    if let Some(p) = port {
        if p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    name == "127.0.0.1" || name.eq_ignore_ascii_case("localhost")
}

fn parse_query(q: &str) -> Vec<(&str, &str)> {
    q.split('&')
        .filter(|s| !s.is_empty())
        .map(|p| p.split_once('=').unwrap_or((p, "")))
        .collect()
}

fn build_config(pairs: &[(&str, &str)], skip: &[&str]) -> Result<Config, String> {
    let mut cfg = Config::default();
    for (k, v) in pairs {
        if skip.contains(k) {
            continue;
        }
        cfg.set(k, v)?;
    }
    cfg.validate()?;
    Ok(cfg)
}

fn config_body() -> String {
    let items: Vec<String> = knobs()
        .iter()
        .map(|k| {
            format!(
                "{{\"key\":\"{}\",\"label\":\"{}\",\"min\":{},\"max\":{},\"default\":{}}}",
                k.key, k.label, k.min, k.max, k.default
            )
        })
        .collect();
    format!("{{\"knobs\":[{}],\"max_runs\":{}}}", items.join(","), MAX_RUNS)
}

fn static_file(root: &Path, name: &str, ct: &'static str) -> Response {
    match std::fs::read(root.join(name)) {
        Ok(b) => Response { status: 200, content_type: ct, body: b },
        Err(_) => err(404, "not found"),
    }
}

pub fn route(root: &Path, method: &str, target: &str, host: Option<&str>) -> Response {
    if !host_ok(host) {
        return err(403, "forbidden host");
    }
    if method != "GET" {
        return err(405, "method not allowed");
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let pairs = parse_query(query);
    match path {
        "/" => static_file(root, "index.html", "text/html; charset=utf-8"),
        "/data/run.js" => static_file(root, "data/run.js", "application/javascript; charset=utf-8"),
        "/data/stats.js" => static_file(root, "data/stats.js", "application/javascript; charset=utf-8"),
        "/api/config" => json_resp(200, config_body()),
        "/api/run" => {
            let mut seed: u64 = 1;
            for (k, v) in &pairs {
                if *k == "seed" {
                    seed = match v.parse() {
                        Ok(s) => s,
                        Err(_) => return err(400, "seed: not a valid u64"),
                    };
                }
            }
            match build_config(&pairs, &["seed"]) {
                Ok(cfg) => json_resp(200, run_body(&simulate_with(seed, &cfg))),
                Err(e) => err(400, &e),
            }
        }
        "/api/batch" => {
            let mut runs: u32 = 2000;
            for (k, v) in &pairs {
                if *k == "runs" {
                    runs = match v.parse() {
                        Ok(r) if (1..=MAX_RUNS).contains(&r) => r,
                        _ => return err(400, "runs: must be an integer in 1..=20000"),
                    };
                }
            }
            match build_config(&pairs, &["runs"]) {
                Ok(cfg) => {
                    let t = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
                    json_resp(200, stats_body(&batch_with(runs, t, &cfg)))
                }
                Err(e) => err(400, &e),
            }
        }
        _ => err(404, "not found"),
    }
}

pub fn bind(port: u16) -> std::io::Result<TcpListener> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, port))
}

fn reason(s: u16) -> &'static str {
    match s {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    }
}

fn write_resp(stream: &mut TcpStream, r: &Response) {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        r.status, reason(r.status), r.content_type, r.body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&r.body);
    let _ = stream.flush();
}
