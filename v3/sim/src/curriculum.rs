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

/// Kind of opponent a trainee can meet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OppKind {
    Dummy,
    /// SkillBot tier 1 to 3 (engaging scripted opponents).
    Skill(u8),
    /// A member of the opposite co-evolving population.
    SelfPlay,
    /// A hall-of-fame champion.
    Hof,
}

/// Opponent mix by generation; weights are each above zero and sum to 1.
/// A (0..29): Dummy .7, tier 1 .3. B (30..79): self-play .3 with tier 1 .7 until 55, then tier 2 .4 and tier 3 .3.
/// C (80+): hall of fame .25, self-play .3, tier 2 .15, tier 3 .3 (self-play breeds mutual passivity, so it is the minority).
pub fn opponent_mix(gen: u32) -> Vec<(OppKind, f32)> {
    match stage_for_gen(gen) {
        Stage::A => vec![(OppKind::Dummy, 0.7), (OppKind::Skill(1), 0.3)],
        Stage::B if gen < 55 => vec![(OppKind::SelfPlay, 0.3), (OppKind::Skill(1), 0.7)],
        Stage::B => vec![(OppKind::SelfPlay, 0.3), (OppKind::Skill(2), 0.4), (OppKind::Skill(3), 0.3)],
        Stage::C => vec![(OppKind::Hof, 0.25), (OppKind::SelfPlay, 0.3), (OppKind::Skill(2), 0.15), (OppKind::Skill(3), 0.3)],
    }
}

/// Pick a kind from the mix with a uniform draw in 0..1.
pub fn pick_kind(mix: &[(OppKind, f32)], u: f32) -> OppKind {
    let mut acc = 0.0;
    for (k, w) in mix {
        acc += w;
        if u < acc {
            return *k;
        }
    }
    mix.last().map(|m| m.0).unwrap_or(OppKind::Dummy)
}
