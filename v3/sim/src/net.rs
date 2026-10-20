//! Policy network: (N_IN + N_CTX) -> N_HID (tanh) -> N_OUT (tanh).
//! The first N_CTX hidden units are fed back as context inputs on the next step.
//! Outputs (19): 14 move logits, 4 move params (reach, speed, arc, delay), 1 drive; all tanh, see `decode`.
//! Weight layout: W1[N_HID*(N_IN+N_CTX)], b1[N_HID], W2[N_OUT*N_HID], b2[N_OUT].
use crate::genome::Genome;
use crate::moveset::{MoveId, MoveParams, N_MOVES};
use crate::sense::N_IN;

pub const N_OUT: usize = 19;
/// A non-Idle move is chosen only if its logit exceeds Idle's by this much.
pub const MOVE_MARGIN: f32 = 0.05;
pub const N_HID: usize = 32;
pub const N_CTX: usize = 8;
pub const N_PARAMS: usize = N_HID * (N_IN + N_CTX) + N_HID + N_OUT * N_HID + N_OUT;

/// Rational tanh (Lambert continued fraction, 7th order), clamped to [-1, 1].
#[inline(always)]
pub fn fast_tanh(x: f32) -> f32 {
    let x = x.clamp(-5.0, 5.0);
    let x2 = x * x;
    let p = x * (135135.0 + x2 * (17325.0 + x2 * (378.0 + x2)));
    let q = 135135.0 + x2 * (62370.0 + x2 * (3150.0 + x2 * 28.0));
    (p / q).clamp(-1.0, 1.0)
}

#[derive(Clone, Debug)]
pub struct Net {
    w1: Vec<f32>,
    b1: Vec<f32>,
    w2: Vec<f32>,
    b2: Vec<f32>,
    ctx: [f32; N_CTX],
}

impl Net {
    pub fn from_genome(g: &Genome) -> Net {
        assert_eq!(g.weights.len(), N_PARAMS, "weights length");
        let w = &g.weights;
        let n1 = N_HID * (N_IN + N_CTX);
        Net {
            w1: w[..n1].to_vec(),
            b1: w[n1..n1 + N_HID].to_vec(),
            w2: w[n1 + N_HID..n1 + N_HID + N_OUT * N_HID].to_vec(),
            b2: w[n1 + N_HID + N_OUT * N_HID..].to_vec(),
            ctx: [0.0; N_CTX],
        }
    }

    pub fn reset(&mut self) {
        self.ctx = [0.0; N_CTX];
    }

    pub fn forward(&mut self, x: &[f32; N_IN]) -> [f32; N_OUT] {
        let mut h = [0.0f32; N_HID];
        for (j, hj) in h.iter_mut().enumerate() {
            let row = &self.w1[j * (N_IN + N_CTX)..(j + 1) * (N_IN + N_CTX)];
            let mut s = self.b1[j];
            for i in 0..N_IN {
                s += row[i] * x[i];
            }
            for k in 0..N_CTX {
                s += row[N_IN + k] * self.ctx[k];
            }
            *hj = fast_tanh(s);
        }
        self.ctx.copy_from_slice(&h[..N_CTX]);
        let mut out = [0.0f32; N_OUT];
        for (o, ov) in out.iter_mut().enumerate() {
            let row = &self.w2[o * N_HID..(o + 1) * N_HID];
            let mut s = self.b2[o];
            for j in 0..N_HID {
                s += row[j] * h[j];
            }
            *ov = fast_tanh(s);
        }
        out
    }
}

/// A decoded net output: the requested move, its params and the footwork drive.
#[derive(Clone, Copy, Debug)]
pub struct Decoded {
    pub mv: MoveId,
    pub params: MoveParams,
    pub drive: f32,
}

/// Argmax over the 14 logits (ties to the lowest index); a non-Idle move needs `MOVE_MARGIN` over Idle.
pub fn decode(out: &[f32; N_OUT]) -> Decoded {
    let mut best = 0;
    for i in 1..N_MOVES {
        if out[i] > out[best] {
            best = i;
        }
    }
    let mv = if best != 0 && out[best] > out[0] + MOVE_MARGIN { MoveId::from_index(best) } else { MoveId::Idle };
    let u = |v: f32| ((v + 1.0) * 0.5).clamp(0.0, 1.0);
    Decoded {
        mv,
        params: MoveParams { reach: u(out[14]), speed: u(out[15]), arc: u(out[16]), delay: u(out[17]) },
        drive: out[18].clamp(-1.0, 1.0),
    }
}
