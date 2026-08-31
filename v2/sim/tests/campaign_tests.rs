// Assumed API: campaign::{Library::new(Vec<Genome>), Library::pick(i)->&Genome, MapCfg::default(),
// CampaignResult::end_count(), Encounter{kind, zone, tile, arena_seed, match_seed, monster_index, result},
// tier_stats(zone_tier, boss)->TierStats{hp,damage}, Outcome{Won,Died,Capped}, EncounterKind{Grass,Elite,Boss}}.
// Replay rule: hero = library.pick(0) here (single-genome libraries), monster = monster_lib.pick(e.monster_index).
use arena_sim::campaign::{run, tier_stats, EncounterKind, Library, MapCfg, Outcome as CO};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;
