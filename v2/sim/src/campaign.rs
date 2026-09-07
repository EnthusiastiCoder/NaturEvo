//! Campaign: the hero walks camp to boss over the jungle map; each encounter is an arena duel.
use crate::config::ArenaConfig;
use crate::genome::Genome;
use crate::jungle::{Map, Tile};
use crate::matchup::{play, MatchResult, Outcome as MOutcome};
use crate::rng::Rng;
use crate::world::Arena;

const CAMPAIGN_SALT: u64 = 0x4341_4D50_4149_474E;

/// A set of genomes to draw fighters from.
#[derive(Clone, Debug)]
pub struct Library {
    genomes: Vec<Genome>,
}
