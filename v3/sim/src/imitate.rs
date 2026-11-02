//! Imitation: distil a scripted style teacher into a net (supervised, from the net's own sensed inputs).
//! The loss is a softmax cross-entropy over the 14 move outputs plus squared error on the four move params (only when
//! a move is asked for) and on the footwork drive. The recurrent context is carried through each fight in order and
//! treated as a constant input (truncated backprop). Later rounds (DAgger) let the current student fight while the
//! teacher watches and labels the states the student actually reaches.
use crate::arena::Arena;
use crate::duel::{Brain, Match, Sample};
use crate::genome::Genome;
use crate::net::{fast_tanh, N_CTX, N_HID, N_OUT, N_PARAMS};
use crate::moveset::N_MOVES;
use crate::rng::Rng;
use crate::robot::Role;
use crate::sense::N_IN;
use crate::stylebot::StyleId;

/// Softmax temperature on the tanh move outputs.
const K_LOGIT: f32 = 4.0;
/// Extra weight on samples that ask for a move (Idle dominates the data).
const W_MOVE: f32 = 2.5;
const W_PARAM: f32 = 1.0;
const W_DRIVE: f32 = 1.0;
/// Longest fight sequence kept (decision steps).
const MAX_SEQ: usize = 1200;

/// Per-input standardisation used while fitting; folded into the first layer afterwards so the net takes raw inputs.
#[derive(Clone)]
pub struct Norm {
    mu: Vec<f32>,
    sd: Vec<f32>,
}

impl Norm {
    pub fn from_data(d: &Dataset) -> Norm {
        let (mut sum, mut sq, mut n) = (vec![0.0f64; N_IN], vec![0.0f64; N_IN], 0.0f64);
        for seq in &d.seqs {
            for s in seq {
                for i in 0..N_IN {
                    let v = s.x[i] as f64;
                    sum[i] += v;
                    sq[i] += v * v;
                }
                n += 1.0;
            }
        }
        let n = n.max(1.0);
        let mu: Vec<f32> = sum.iter().map(|v| (v / n) as f32).collect();
        let sd: Vec<f32> = (0..N_IN).map(|i| (((sq[i] / n) - (sum[i] / n).powi(2)).max(0.0).sqrt() as f32).max(SD_FLOOR)).collect();
        Norm { mu, sd }
    }
    fn apply(&self, x: &[f32]) -> [f32; N_IN] {
        let mut o = [0.0f32; N_IN];
        for i in 0..N_IN {
            o[i] = (x[i] - self.mu[i]) / self.sd[i];
        }
        o
    }
    /// Weights acting on raw inputs that reproduce `w` acting on standardised inputs.
    pub fn fold(&self, w: &[f32]) -> Vec<f32> {
        let mut o = w.to_vec();
        let n1 = w1_len();
        for j in 0..N_HID {
            let base = j * (N_IN + N_CTX);
            let mut shift = 0.0;
            for i in 0..N_IN {
                let wi = w[base + i] / self.sd[i];
                o[base + i] = wi;
                shift += wi * self.mu[i];
            }
            o[n1 + j] -= shift;
        }
        o
    }
}

/// Smallest input spread used when standardising (keeps folded weights bounded).
const SD_FLOOR: f32 = 0.03;

pub struct Dataset {
    pub seqs: Vec<Vec<Sample>>,
}

impl Dataset {
    pub fn len(&self) -> usize {
        self.seqs.iter().map(|s| s.len()).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.seqs.is_empty()
    }
    pub fn extend(&mut self, o: Dataset) {
        self.seqs.extend(o.seqs);
    }
}
