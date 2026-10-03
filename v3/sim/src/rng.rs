//! Deterministic splitmix64 RNG. No platform-dependent behaviour.

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}
