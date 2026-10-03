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
