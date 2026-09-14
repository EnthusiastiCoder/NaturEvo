//! Policy network: N_IN -> H (tanh, recurrent) -> N_OUT (tanh).
//! Weight layout: W1[H*N_IN], R[H*H], b1[H], W2[N_OUT*H], b2[N_OUT].
use crate::fighter::Action;
use crate::genome::Genome;
use crate::sense::N_IN;

pub const N_OUT: usize = 7;
pub const N_HID: usize = 16;
pub const N_PARAMS: usize = N_HID * N_IN + N_HID * N_HID + N_HID + N_OUT * N_HID + N_OUT;

/// Rational tanh (Lambert continued fraction, 7th order), branch-free so it vectorises.
/// Abs error is below 1e-4 over the range, and the output is clamped to [-1, 1].
#[inline(always)]
pub fn fast_tanh(x: f32) -> f32 {
    let x = x.clamp(-5.0, 5.0);
    let x2 = x * x;
    let p = x * (135135.0 + x2 * (17325.0 + x2 * (378.0 + x2)));
    let q = 135135.0 + x2 * (62370.0 + x2 * (3150.0 + x2 * 28.0));
    (p / q).clamp(-1.0, 1.0)
}

/// Weights are stored transposed (input-major) so the inner loops run over the
/// hidden units and vectorise. Per-unit summation order matches the genome layout
/// (bias, inputs in order, recurrent in order), so results are bit-identical.
#[derive(Clone, Debug)]
pub struct Net {
    w1t: [[f32; N_HID]; N_IN],
    rt: [[f32; N_HID]; N_HID],
    b1: [f32; N_HID],
    w2t: [[f32; N_OUT]; N_HID],
    b2: [f32; N_OUT],
    h: [f32; N_HID],
}

impl Net {
    pub fn from_genome(g: &Genome) -> Net {
        let mut w = g.weights.clone();
        w.resize(N_PARAMS, 0.0);
        let r_off = N_HID * N_IN;
        let b1o = r_off + N_HID * N_HID;
        let w2o = b1o + N_HID;
        let b2o = w2o + N_OUT * N_HID;
        let mut n = Net {
            w1t: [[0.0; N_HID]; N_IN],
            rt: [[0.0; N_HID]; N_HID],
            b1: [0.0; N_HID],
            w2t: [[0.0; N_OUT]; N_HID],
            b2: [0.0; N_OUT],
            h: [0.0; N_HID],
        };
        for j in 0..N_HID {
            n.b1[j] = w[b1o + j];
            for i in 0..N_IN {
                n.w1t[i][j] = w[j * N_IN + i];
            }
            for k in 0..N_HID {
                n.rt[k][j] = w[r_off + j * N_HID + k];
            }
        }
        for o in 0..N_OUT {
            n.b2[o] = w[b2o + o];
            for j in 0..N_HID {
                n.w2t[j][o] = w[w2o + o * N_HID + j];
            }
        }
        n
    }

    pub fn reset(&mut self) {
        self.h = [0.0; N_HID];
    }

    pub fn forward(&mut self, x: &[f32; N_IN]) -> Action {
        let mut nh = self.b1;
        for (xi, col) in x.iter().zip(self.w1t.iter()) {
            for j in 0..N_HID {
                nh[j] += col[j] * xi;
            }
        }
        for (hk, col) in self.h.iter().zip(self.rt.iter()) {
            for j in 0..N_HID {
                nh[j] += col[j] * hk;
            }
        }
        for v in nh.iter_mut() {
            *v = fast_tanh(*v);
        }
        self.h = nh;
        let mut out = self.b2;
        for (hj, col) in nh.iter().zip(self.w2t.iter()) {
            for o in 0..N_OUT {
                out[o] += col[o] * hj;
            }
        }
        // Only the three analogue outputs need squashing; the rest are used by sign alone
        // and tanh preserves sign.
        for v in out[..3].iter_mut() {
            *v = fast_tanh(*v);
        }
        Action {
            fwd: out[0],
            strafe: out[1],
            turn: out[2],
            attack: out[3] > 0.0,
            block: out[4] > 0.0,
            dodge: out[5] > 0.0,
        }
    }
}
