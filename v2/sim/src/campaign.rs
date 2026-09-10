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
    /// Load trained champions of one role ("hero" or "monster") from a `train` output dir, newest
    /// generation first, keeping at most `keep`. Index 0 is therefore the latest champion.
    pub fn from_dir(dir: &str, role: &str, keep: usize) -> Result<Library, String> {
        let prefix = format!("champion_{}_gen_", role);
        let mut found: Vec<(u32, std::path::PathBuf)> = Vec::new();
        for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {}", dir, e))? {
            let path = e.map_err(|e| e.to_string())?.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if let Some(g) = name.strip_prefix(&prefix).and_then(|r| r.strip_suffix(".json")).and_then(|g| g.parse::<u32>().ok()) {
                found.push((g, path));
            }
        }
        found.sort_by(|a, b| b.0.cmp(&a.0));
        found.truncate(keep.max(1));
        let mut gs = Vec::new();
        for (_, p) in &found {
            let t = std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p.display(), e))?;
            gs.push(Genome::from_json(&t)?);
        }
        if gs.is_empty() {
            return Err(format!("no {} champions in {}", role, dir));
        }
        Ok(Library::new(gs))
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

#[derive(Clone, Copy, Debug)]
pub struct TierStats {
    pub hp: f32,
    pub damage: f32,
}

/// Monster strength by zone tier (0 to 3). Recorded per encounter; duels themselves use
/// the shared arena rules so every log stays replayable from its seeds.
pub fn tier_stats(tier: u32, boss: bool) -> TierStats {
    let t = tier as f32;
    let (hp, damage) = (100.0 * (1.0 + 0.25 * t), 12.0 * (1.0 + 0.15 * t));
    if boss {
        TierStats { hp: hp * 1.5, damage: damage * 1.2 }
    } else {
        TierStats { hp, damage }
    }
}

#[derive(Clone, Debug)]
pub struct Encounter {
    pub kind: EncounterKind,
    pub zone: u32,
    pub tile: (i32, i32),
    pub arena_seed: u64,
    pub match_seed: u64,
    pub monster_index: usize,
    pub stats: TierStats,
    pub result: MatchResult,
}

#[derive(Clone, Debug)]
pub struct CampaignResult {
    pub map: Map,
    pub path: Vec<(i32, i32)>,
    pub steps: usize,
    pub encounters: Vec<Encounter>,
    pub outcome: Outcome,
    ends: u32,
}

impl CampaignResult {
    /// Number of run-end events recorded (exactly one for a finished run).
    pub fn end_count(&self) -> u32 {
        self.ends
    }
}

/// Walk the A* route from camp to boss. Grass tiles may trigger a duel, the first step into each
/// new zone triggers an elite duel and the boss tile a boss duel. With `permadeath` the first
/// hero loss ends the run; without it the hero keeps walking.
pub fn run(seed: u64, cfg: MapCfg, heroes: &Library, monsters: &Library, permadeath: bool) -> CampaignResult {
    let map = Map::generate(seed, cfg.size);
    let path = map.astar(map.camp(), map.boss()).expect("camp reaches boss");
    let acfg = ArenaConfig::default();
    let mut rng = Rng::new(seed ^ CAMPAIGN_SALT);
    let mut encounters: Vec<Encounter> = Vec::new();
    let mut seen_zones = vec![false; 9];
    let mut steps = 0usize;
    let mut outcome = Outcome::Capped;
    let boss = map.boss();
    for &tile in path.iter().skip(1) {
        steps += 1;
        let zone = map.zone_of(tile.0 as usize, tile.1 as usize);
        let first_visit = !seen_zones[zone];
        seen_zones[zone] = true;
        let kind = if tile == boss {
            Some(EncounterKind::Boss)
        } else if first_visit && zone != 0 {
            Some(EncounterKind::Elite)
        } else if map.tile_at(tile) == Some(Tile::Grass) && rng.f32() < cfg.grass_chance {
            Some(EncounterKind::Grass)
        } else {
            None
        };
        let Some(kind) = kind else { continue };
        if encounters.len() >= cfg.max_encounters && kind != EncounterKind::Boss {
            break;
        }
        let arena_seed = rng.next_u64();
        let match_seed = rng.next_u64();
        let monster_index = (rng.next_u64() % monsters.len() as u64) as usize;
        let tier = map.zone_tier(tile);
        let result = play(
            Arena::generate(arena_seed, &acfg),
            heroes.pick(0).clone(),
            monsters.pick(monster_index).clone(),
            match_seed,
            &acfg,
        );
        let lost = result.outcome != MOutcome::HeroWin;
        encounters.push(Encounter {
            kind,
            zone: zone as u32,
            tile,
            arena_seed,
            match_seed,
            monster_index,
            stats: tier_stats(tier, kind == EncounterKind::Boss),
            result,
        });
        if lost && permadeath {
            outcome = Outcome::Died;
            break;
        }
        if kind == EncounterKind::Boss {
            outcome = if lost { Outcome::Died } else { Outcome::Won };
            break;
        }
    }
    CampaignResult { map, path, steps, encounters, outcome, ends: 1 }
}

/// Arena obstacles as `{"w","h","blocked":[rows of '#'/'.']}` for the viewer.
pub fn arena_json(a: &Arena) -> String {
    let (w, h) = a.size();
    let t = a.tiles();
    let rows: Vec<String> = (0..h)
        .map(|y| format!("\"{}\"", (0..w).map(|x| if t[y * w + x] { '#' } else { '.' }).collect::<String>()))
        .collect();
    format!("{{\"w\":{},\"h\":{},\"blocked\":[{}]}}", w, h, rows.join(","))
}
}
