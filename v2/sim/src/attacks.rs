//! Attack definitions: hero moves and monster species movesets (data tables).
use crate::status::Status;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Hero,
    Minion,
    Rogue,
    Warrior,
    Mage,
}
