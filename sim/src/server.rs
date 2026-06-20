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
