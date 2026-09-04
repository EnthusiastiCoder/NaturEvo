//! Arena grid: blocked tiles, spawns, reachability.
use crate::config::ArenaConfig;
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Arena {
    w: usize,
    h: usize,
    blocked: Vec<bool>,
    spawns: [(f32, f32); 2],
}
