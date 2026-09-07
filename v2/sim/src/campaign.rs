//! Campaign: the hero walks camp to boss over the jungle map; each encounter is an arena duel.
use crate::config::ArenaConfig;
use crate::genome::Genome;
use crate::jungle::{Map, Tile};
use crate::matchup::{play, MatchResult, Outcome as MOutcome};
use crate::rng::Rng;
use crate::world::Arena;
