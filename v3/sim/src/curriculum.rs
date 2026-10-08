//! Training curriculum by generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Gens below 30: mostly a standing dummy, some scripted attacks; learn to stand, reach and hit.
    A,
    /// Gens 30 to 79: scripted attacker (forces a defence signal) plus self-play.
    B,
    /// Gens 80 and up: self-play, hall of fame and one scripted attacker.
    C,
}

pub fn stage_for_gen(gen: u32) -> Stage {
    if gen < 30 {
        Stage::A
    } else if gen < 80 {
        Stage::B
    } else {
        Stage::C
    }
}

pub fn hof_enabled(s: Stage) -> bool {
    s == Stage::C
}

pub fn opponents_per_individual(s: Stage) -> usize {
    if s == Stage::A {
        2
    } else {
        3
    }
}
