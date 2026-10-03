//! Policy network: (N_IN + N_CTX) -> N_HID (tanh) -> N_OUT (tanh).
//! The first N_CTX hidden units are fed back as context inputs on the next step.
//! Outputs: 12 normalised joint targets (-1..1) and 1 drive (-1..1).
//! Weight layout: W1[N_HID*(N_IN+N_CTX)], b1[N_HID], W2[N_OUT*N_HID], b2[N_OUT].
use crate::genome::Genome;
use crate::sense::N_IN;

pub const N_OUT: usize = 13;
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
