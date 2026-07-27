//! Minimal JSON parser for tests (zero dependencies). Used to read run_body / map_body output.
#![allow(dead_code)]
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
