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

impl Library {
    pub fn new(genomes: Vec<Genome>) -> Library {
        assert!(!genomes.is_empty(), "Library needs at least one genome");
        Library { genomes }
    }
    pub fn len(&self) -> usize {
        self.genomes.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    pub fn pick(&self, i: usize) -> &Genome {
        &self.genomes[i % self.genomes.len()]
    }
}

#[derive(Clone, Debug)]
pub struct MapCfg {
    pub size: usize,
    pub grass_chance: f32,
    pub max_encounters: usize,
}

impl Default for MapCfg {
    fn default() -> Self {
        MapCfg { size: 40, grass_chance: 0.2, max_encounters: 12 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterKind {
    Grass,
    Elite,
    Boss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won,
    Died,
    Capped,
}
