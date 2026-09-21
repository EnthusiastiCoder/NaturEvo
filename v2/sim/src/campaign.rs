//! Campaign: the hero walks camp to boss over the jungle map; each encounter is an arena duel.
use crate::config::ArenaConfig;
use crate::genome::Genome;
use crate::jungle::{Map, Tile};
use crate::attacks::{Rank, Species};
use crate::fighter::Role;
use crate::matchup::{play_with_setup, MatchResult, Outcome as MOutcome, Setup};
use crate::rng::Rng;
use crate::world::Arena;

/// Library screen limits and panel size (2 opponents x 4 seeds = 8 fights).
const MAX_BLOCK_UNSEEN: f32 = 0.10;
const MAX_BLOCK: f32 = 0.25;
const MAX_DRAW: f32 = 0.25;
const SCREEN_SEEDS: u64 = 4;

/// Fixed opponent panel of `role`: one that stands still and one that walks up and swings.
fn screen_panel(role: Role) -> Vec<Genome> {
    use crate::net::{N_OUT, N_PARAMS};
    let bias = |v: &[(usize, f32)]| {
        let mut w = vec![0.0f32; N_PARAMS];
        for (i, x) in v {
            w[N_PARAMS - N_OUT + i] = *x;
        }
        Genome { role, weights: w, generation: 0, fitness: 0.0 }
    };
    vec![bias(&[(3, -1.0), (4, -1.0), (5, -1.0)]), bias(&[(0, 1.0), (3, 1.0), (4, -1.0)])]
}

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
    /// generation first, keeping at most `keep`. Champions that fail the behaviour screen
    /// (turtling, stalemating) are dropped. Index 0 is therefore the latest passing champion.
    pub fn from_dir(dir: &str, role: &str, keep: usize) -> Result<Library, String> {
        Library::from_dir_opts(dir, role, keep, true)
    }

    /// `from_dir` with the screen switched on or off.
    pub fn from_dir_opts(dir: &str, role: &str, keep: usize, screen: bool) -> Result<Library, String> {
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
        let keep = keep.max(1);
        let r = if role == "monster" { Role::Monster } else { Role::Hero };
        let panel = screen_panel(r.other());
        let mut gs = Vec::new();
        let mut report: Vec<String> = Vec::new();
        for (g, p) in &found {
            if gs.len() >= keep {
                break;
            }
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

/// Starting hp for the monster by encounter kind. The hero always starts at full hp.
pub fn monster_hp_for(kind: EncounterKind) -> f32 {
    match kind {
        EncounterKind::Grass => GRASS_HP,
        EncounterKind::Elite => ELITE_HP,
        EncounterKind::Boss => 100.0,
    }
}
const GRASS_HP: f32 = 85.0;
const ELITE_HP: f32 = 96.0;

#[derive(Clone, Debug)]
pub struct Encounter {
    pub kind: EncounterKind,
    pub zone: u32,
    pub tile: (i32, i32),
    pub arena_seed: u64,
    pub match_seed: u64,
    pub monster_index: usize,
    pub stats: TierStats,
    /// Monster starting hp in the duel (wild monsters are handicapped, the boss is at full strength).
    pub monster_hp: f32,
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
        // Library index 0 is the latest champion; higher indices are older and weaker. Grass meets
        // the oldest champions, elites the middle ones, the boss always the latest.
        let n = monsters.len();
        let roll = rng.f32();
        let (f_lo, f_hi) = match kind {
            EncounterKind::Boss => (0.0, 0.0),
            EncounterKind::Elite => (0.2, 0.65),
            EncounterKind::Grass => (0.65, 1.0),
        };
        let f = f_lo + (f_hi - f_lo) * roll;
        let mut monster_index = (f * (n - 1) as f32).round() as usize;
        if kind == EncounterKind::Elite && n > 1 {
            monster_index = monster_index.max(1);
        }
        let tier = map.zone_tier(tile);
        let monster_hp = monster_hp_for(kind);
        let result = play_with_hp(
            Arena::generate(arena_seed, &acfg),
            heroes.pick(0).clone(),
            monsters.pick(monster_index).clone(),
            match_seed,
            &acfg,
            [acfg.max_hp, monster_hp],
        );
        let lost = result.outcome != MOutcome::HeroWin;
        encounters.push(Encounter {
            kind,
            zone: zone as u32,
            tile,
            arena_seed,
            match_seed,
            monster_index,
            monster_hp,
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

/// Fight log JSON plus `arena`, `seed` and `monster_index` fields.
pub fn fight_json(e: &Encounter) -> String {
    let log = e.result.log.to_json();
    let arena = Arena::generate(e.arena_seed, &ArenaConfig::default());
    let body = log.strip_suffix('}').unwrap_or(&log);
    format!("{},\"arena\":{},\"seed\":{},\"monster_index\":{}}}", body, arena_json(&arena), e.match_seed, e.monster_index)
}

/// Run JSON for the viewer. With `fights_dir` set, fights are written there as `fight_N.json`
/// and referenced by relative URL; otherwise they are inlined.
pub fn run_json(r: &CampaignResult, fights_dir: Option<&str>, run_file: Option<&str>) -> Result<String, String> {
    // fight URLs are stored relative to the run file's directory so the viewer resolves them
    let prefix = match (fights_dir, run_file) {
        (Some(d), Some(f)) => {
            let parent = std::path::Path::new(f).parent().unwrap_or(std::path::Path::new(""));
            let rel = std::path::Path::new(d).strip_prefix(parent).unwrap_or(std::path::Path::new(d));
            let s = rel.to_string_lossy().replace("\\", "/");
            if s.is_empty() { String::new() } else { format!("{}/", s.trim_end_matches('/')) }
        }
        _ => String::new(),
    };
    let mut encs = Vec::new();
    for (i, e) in r.encounters.iter().enumerate() {
        let fj = fight_json(e);
        let fight = match fights_dir {
            Some(d) => {
                std::fs::create_dir_all(d).map_err(|x| format!("{}: {}", d, x))?;
                let name = format!("fight_{}.json", i);
                std::fs::write(std::path::Path::new(d).join(&name), &fj).map_err(|x| x.to_string())?;
                format!("\"{}{}\"", prefix, name)
            }
            None => fj,
        };
        let kind = format!("{:?}", e.kind).to_lowercase();
        encs.push(format!(
            "{{\"kind\":\"{}\",\"zone\":{},\"tile\":[{},{}],\"monster_index\":{},\"result\":\"{:?}\",\"fight\":{}}}",
            kind, e.zone, e.tile.0, e.tile.1, e.monster_index, e.result.outcome, fight
        ));
    }
    let path: Vec<String> = r.path.iter().map(|p| format!("[{},{}]", p.0, p.1)).collect();
    Ok(format!(
        "{{\"map\":{},\"path\":[{}],\"encounters\":[{}],\"outcome\":\"{:?}\"}}",
        r.map.to_json(), path.join(","), encs.join(","), r.outcome
    ))
}
