//! Deterministic tile map: zone biomes (Voronoi), grass encounter tiles, walls and A* pathing.
use crate::rng::Rng;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
