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
