//! Policy network (WS2 minimal single linear layer; WS4 owns the real design).
use crate::fighter::Action;
use crate::genome::Genome;
use crate::sense::N_IN;

pub const N_OUT: usize = 7;

#[derive(Clone, Debug)]
pub struct Net {
    w: Vec<f32>,
}

impl Net {
    pub fn from_genome(g: &Genome) -> Net {
        Net { w: g.weights.clone() }
    }
