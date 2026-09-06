//! Policy network: N_IN -> H (tanh, recurrent) -> N_OUT (tanh).
//! Weight layout: W1[H*N_IN], R[H*H], b1[H], W2[N_OUT*H], b2[N_OUT].
use crate::fighter::Action;
use crate::genome::Genome;
use crate::sense::N_IN;

pub const N_OUT: usize = 7;
pub const N_HID: usize = 16;
pub const N_PARAMS: usize = N_HID * N_IN + N_HID * N_HID + N_HID + N_OUT * N_HID + N_OUT;

#[derive(Clone, Debug)]
pub struct Net {
    w: Vec<f32>,
    h: [f32; N_HID],
}

impl Net {
    pub fn from_genome(g: &Genome) -> Net {
        let mut w = g.weights.clone();
        w.resize(N_PARAMS, 0.0);
        Net { w, h: [0.0; N_HID] }
    }

    pub fn reset(&mut self) {
        self.h = [0.0; N_HID];
    }

    pub fn forward(&mut self, x: &[f32; N_IN]) -> Action {
        let w = &self.w;
        let r_off = N_HID * N_IN;
        let b1 = r_off + N_HID * N_HID;
        let w2 = b1 + N_HID;
        let b2 = w2 + N_OUT * N_HID;
        let mut nh = [0.0f32; N_HID];
        for (j, slot) in nh.iter_mut().enumerate() {
            let mut s = w[b1 + j];
            for (i, xi) in x.iter().enumerate() {
                s += w[j * N_IN + i] * xi;
            }
            for (k, hk) in self.h.iter().enumerate() {
                s += w[r_off + j * N_HID + k] * hk;
            }
            *slot = s.tanh();
        }
        self.h = nh;
        let mut out = [0.0f32; N_OUT];
        for (o, slot) in out.iter_mut().enumerate() {
            let mut s = w[b2 + o];
            for (j, hj) in nh.iter().enumerate() {
                s += w[w2 + o * N_HID + j] * hj;
            }
            *slot = s.tanh();
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
