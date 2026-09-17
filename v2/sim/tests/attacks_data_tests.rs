// WS1 move tables and log contract. Plan inconsistency: plan says total frames 14..46, hero Light is 4+3+6 = 13, so bound is 13..=46.
mod common;
use arena_sim::attacks::{moves, Move, Rank, Species};
use arena_sim::status::Status;
use common::*;
use std::f32::consts::PI;

fn has(m: &Move, s: Status) -> bool { m.effects.iter().any(|e| e.status == s) }
fn eff(m: &Move, s: Status) -> (u32, f32) {
    let e = m.effects.iter().find(|e| e.status == s).unwrap_or_else(|| panic!("{} lacks {:?}", m.name, s));
    (e.duration, e.magnitude)
}
const SPECIES: [Species; 4] = [Species::Minion, Species::Rogue, Species::Warrior, Species::Mage];
