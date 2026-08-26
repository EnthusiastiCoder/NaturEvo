use crate::config::Config;
use crate::data::*;
use crate::map::{Map, Tile};
use crate::rng::Rng;

const MAX_LEVEL: u32 = 40;
const MAX_ROUNDS: u32 = 60;

#[derive(Clone, Debug)]
pub struct HeroSnap {
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub xp: u32,
    pub gold: u32,
    pub potions: u32,
    pub zone: usize,
    pub weapon: usize,
    pub armor: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug)]
pub struct MonSnap {
    pub species: usize,
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub stage: u8,
    pub x: usize,
    pub y: usize,
}

/// A visible roaming NPC (elite or boss) in the hero's zone.
#[derive(Clone, Debug)]
pub struct NpcSnap {
    pub id: String,
    pub kind: &'static str,
    pub species: usize,
    pub level: u32,
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub idx: u32,
    pub kind: &'static str,
    pub text: String,
    pub hero: HeroSnap,
    pub mon: Option<MonSnap>,
    pub npcs: Vec<NpcSnap>,
    /// turn-point waypoints of a "walk" event (empty for every other kind)
    pub path: Vec<(u16, u16)>,
    /// every tile of a "walk" leg in order, excluding the start tile
    pub steps: Vec<(u16, u16)>,
    /// per hero step, the (npc id, x, y) of NPCs that moved on it
    pub nsteps: Vec<Vec<(String, u16, u16)>>,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub seed: u64,
    pub steps: u32,
    pub map_size: u32,
    pub events: Vec<Event>,
    pub won: bool,
    pub deaths: u32,
    pub final_level: u32,
    pub first_win_event: Option<u32>,
    pub end_level: u32,
    pub post_kills: u32,
    pub post_deaths: u32,
    pub max_stage: u8,
    pub capped: bool,
    pub outcome: &'static str,
    pub died_level: Option<u32>,
    pub death_zone: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub steps: u32,
    pub won: bool,
    pub deaths: u32,
    pub final_level: u32,
    pub first_win_event: Option<u32>,
    pub events: u32,
    pub end_level: u32,
    pub post_kills: u32,
    pub post_deaths: u32,
    pub max_stage: u8,
    pub capped: bool,
    pub outcome: &'static str,
    pub died_level: Option<u32>,
    pub death_zone: Option<usize>,
}

impl Run {
    pub fn summary(&self) -> Summary {
        Summary {
            steps: self.steps,
            won: self.won,
            deaths: self.deaths,
            final_level: self.final_level,
            first_win_event: self.first_win_event,
            events: self.events.len() as u32,
            end_level: self.end_level,
            post_kills: self.post_kills,
            post_deaths: self.post_deaths,
            max_stage: self.max_stage,
            capped: self.capped,
            outcome: self.outcome,
            died_level: self.died_level,
            death_zone: self.death_zone,
        }
    }
}

/// Emit an event; in lean mode nothing is formatted or stored.
macro_rules! ev {
    ($s:expr, $kind:expr, $mon:expr, $($arg:tt)*) => {
        if $s.lean {
            $s.emit_lean();
        } else {
            let t = format!($($arg)*);
            $s.emit($kind, t, $mon);
        }
    };
}

/// XP needed to go from `level` to `level + 1`.
pub fn xp_to_next(level: u32) -> u32 {
    8 * level * level + 22 * level + 10
}

/// Pokemon-like stat scaling: grows linearly with level.
pub fn monster_stat(base: f32, level: u32) -> f32 {
    base * (0.6 + 0.116 * level as f32)
}

struct Hero {
    level: u32,
    hp: i32,
    max_hp: i32,
    xp: u32,
    gold: u32,
    potions: u32,
    zone: usize,
    weapon: usize,
    armor: usize,
    x: usize,
    y: usize,
}

impl Hero {
    fn max_hp_for(cfg: &Config, level: u32) -> i32 {
        cfg.hero_hp_base + cfg.hero_hp_per_level * level as i32
    }
    fn atk(&self, cfg: &Config) -> f32 {
        8.0 + cfg.hero_atk_per_level * self.level as f32 + WEAPONS[self.weapon].2 as f32
    }
    fn def(&self, cfg: &Config) -> f32 {
        3.0 + cfg.hero_def_per_level * self.level as f32 + ARMORS[self.armor].2 as f32
    }
    fn spd(&self) -> f32 {
        10.0 + self.level as f32 * 0.5
    }
}

struct Mon {
    species: usize,
    level: u32,
    hp: i32,
    max_hp: i32,
    atk: f32,
    def: f32,
    spd: f32,
    behavior: Behavior,
    packed: u32,
    stage: u8,
    x: usize,
    y: usize,
    elite: bool,
}

/// A roaming elite NPC. `dead_until == 0` means alive.
struct Elite {
    zone: usize,
    species: usize,
    level: u32,
    home: (usize, usize),
    pos: (usize, usize),
    dead_until: u32,
    cooldown: u32,
}

const ELITE_RESPAWN_STEPS: u32 = 400;
const ELITE_CHASE_RADIUS: usize = 2;
const ELITE_ROAM: usize = 6;
const NPC_GAP: usize = 4;
const FIGHT_COOLDOWN: u32 = 6;
const ELITE_LEVEL_MARGIN: u32 = 3;
const MAX_WAYPOINTS: usize = 64;
const MAX_LEG_STEPS: usize = 256;
const BOSS_ID: u16 = u16::MAX;
// Hero AI under permadeath: a death is final, so the hero judges each fight before and during it.
// With permadeath off the original thresholds apply.
const HEAL_PCT: i32 = 50;
const RETREAT_PCT: i32 = 80;
const OLD_HEAL_PCT: i32 = 35;
const OLD_RETREAT_PCT: i32 = 50;
/// the hero backs out when the expected damage of finishing a fight exceeds this share of what it can absorb
const RISK_SHARE: f32 = 0.85;
const POST_RISK_SHARE: f32 = 0.6;
/// the hero enters the boss fight only when the expected damage is within this share of its budget
const BOSS_ENTER_SHARE: f32 = 0.75;
/// ...or after this many steps of waiting for that, so a run always ends
const BOSS_WAIT_STEPS: u32 = 40_000;

struct Sim {
    cfg: Config,
    rng: Rng,
    map: Map,
    hero: Hero,
    events: Vec<Event>,
    deaths: u32,
    boss_deaths: u32,
    won: bool,
    done: bool,
    lean: bool,
    count: usize,
    capped: bool,
    died_level: Option<u32>,
    death_zone: Option<usize>,
    ended: bool,
    boss_flees: u32,
    boss_wait: Option<u32>,
    boss_hold: u32,
    flee_state: (u32, usize, usize),
    outcome: &'static str,
    boss_go: bool,
    fight_cd: u32,
    first_win_event: Option<u32>,
    final_level: u32,
    post_kills: u32,
    post_deaths: u32,
    stage: u8,
    steps: u32,
    elites: Vec<Elite>,
    next_respawn: u32,
    boss: (usize, usize),
    goal: usize,
    last_travel: usize,
    leg: Vec<(usize, usize)>,
    leg_dir: (i32, i32),
    leg_tiles: u32,
    next_path: Vec<(u16, u16)>,
    leg_steps: Vec<(u16, u16)>,
    leg_nsteps: Vec<Vec<(u16, u16, u16)>>,
    /// grass tiles per zone, row-major
    zone_grass: Vec<Vec<(usize, usize)>>,
    /// per zone, the index into `zone_grass` where each map row starts (n + 1 entries)
    zone_rows: Vec<Vec<usize>>,
    /// cached camp -> zone entry tile and route, per zone
    outbound: Vec<Option<((usize, usize), Vec<(usize, usize)>)>>,
}

enum Outcome {
    Won,
    Died,
    Fled,
}

/// How a walk (or one step of it) ended.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Walk {
    Arrived,
    Died,
    Fled,
    /// go back to camp (low supplies, or the boss fell)
    Retreat,
    /// the target zone changed
    Retarget,
    Aborted,
}

const CHECK_SUPPLIES: u8 = 1;
const CHECK_TARGET: u8 = 2;

fn damage(rng: &mut Rng, atk: f32, def: f32) -> i32 {
    let base = (atk - def * 0.5).max(atk * 0.1).max(1.0);
    let v = base * (0.85 + 0.3 * rng.f());
    (v as i32).max(1)
}

fn dist(a: (usize, usize), b: (usize, usize)) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

fn pick_weighted(rng: &mut Rng, table: &[(usize, u32)]) -> usize {
    let total: u32 = table.iter().map(|t| t.1).sum();
    let mut r = rng.range(0, total as i32 - 1) as u32;
    for &(sp, w) in table {
        if r < w {
            return sp;
        }
        r -= w;
    }
    table[table.len() - 1].0
}

fn pick_move(rng: &mut Rng, species: usize) -> &'static Move {
    let ms = MOVES[species];
    let total: u32 = ms.iter().map(|m| m.weight).sum();
    let mut r = rng.range(0, total as i32 - 1) as u32;
    for m in ms {
        if r < m.weight {
            return m;
        }
        r -= m.weight;
    }
    &ms[ms.len() - 1]
}

/// Weighted mean move power of a species; damage is normalised by it so move variety
/// changes the shape of a fight, not the species' overall damage output.
fn mean_power(species: usize) -> f32 {
    let ms = MOVES[species];
    let total: u32 = ms.iter().map(|m| m.weight).sum();
    ms.iter().map(|m| m.power * m.weight as f32).sum::<f32>() / total as f32
}

fn step_dir(rng: &mut Rng, p: (usize, usize)) -> (i32, i32) {
    let (dx, dy) = match rng.range(0, 3) {
        0 => (1, 0),
        1 => (-1, 0),
        2 => (0, 1),
        _ => (0, -1),
    };
    (p.0 as i32 + dx, p.1 as i32 + dy)
}

impl Sim {
    fn emit_lean(&mut self) {
        if self.done {
            return;
        }
        self.count += 1;
        if self.count >= self.cfg.event_cap as usize {
            self.done = true;
            self.capped = true;
        }
    }

    /// Whether the hero may back out of this fight. Backing out of the boss is allowed only while it
    /// keeps improving (level, weapon, armor); after `boss_attempts` retreats with no gain it commits.
    fn may_flee(&mut self, boss: bool) -> bool {
        if !boss {
            return true;
        }
        if self.hero.level < MAX_LEVEL {
            // not yet at full strength: back out, then grind a level before trying again
            self.boss_hold = self.hero.level + 1;
            return true;
        }
        let state = (self.hero.level, self.hero.weapon, self.hero.armor);
        if state != self.flee_state {
            self.flee_state = state;
            self.boss_flees = 0;
        }
        if self.boss_flees >= self.cfg.boss_attempts {
            return false;
        }
        self.boss_flees += 1;
        true
    }

    /// Expected damage to finish this fight versus what the hero can absorb (hp plus potions).
    fn too_risky(&self, m: &Mon) -> bool {
        let share = if self.won { POST_RISK_SHARE } else { RISK_SHARE };
        self.risk_ratio(m, self.hero.hp, self.hero.potions) > share
    }

    /// Expected damage taken to finish `m` divided by what the hero can absorb at `hp` with `potions`.
    /// A potion heals half the max hp but costs a turn, so each one is worth heal minus one monster round.
    fn risk_ratio(&self, m: &Mon, hp: i32, potions: u32) -> f32 {
        let h = &self.hero;
        let atk = h.atk(&self.cfg) * 1.25;
        let hit = (atk - m.def * 0.5).max(atk * 0.1).max(1.0);
        let rounds = (m.hp as f32 / hit).ceil().max(1.0);
        let factor = match m.behavior {
            Behavior::Aggressive => 1.25,
            Behavior::Charger => 1.4,
            Behavior::Berserker => 1.3,
            _ => 1.0,
        };
        let matk = m.atk * factor;
        let per_round = (matk - h.def(&self.cfg) * 0.5).max(matk * 0.1).max(1.0);
        let expected = rounds * per_round * 1.15;
        let budget = hp as f32 + potions as f32 * ((h.max_hp / 2) as f32 * 0.9 - per_round).max(0.0);
        expected / budget.max(1.0)
    }

    /// Is the boss fight worth starting now? Judged for a full-hp hero with its current potions.
    fn boss_favourable(&mut self) -> bool {
        let lvl = (ZONES[8].min + ZONES[8].max) / 2;
        let boss = self.map.anchors()[8];
        let m = self.spawn(BOSS, lvl, 0, false, boss);
        self.risk_ratio(&m, self.hero.max_hp, self.hero.potions) <= BOSS_ENTER_SHARE
    }


    /// The hardest single hit this monster can land next round (best move, burst behaviour, high roll).
    fn worst_hit(&self, m: &Mon) -> i32 {
        let best = MOVES[m.species].iter().map(|mv| mv.power).fold(0.0f32, f32::max);
        let mut atk = m.atk * best / mean_power(m.species);
        match m.behavior {
            Behavior::Aggressive => atk *= 1.25,
            Behavior::Berserker => atk *= 1.6,
            Behavior::Charger => atk *= 2.2,
            _ => {}
        }
        let def = self.hero.def(&self.cfg);
        ((atk - def * 0.5).max(atk * 0.1).max(1.0) * 1.15) as i32 + 1
    }

    fn retreat_pct(&self) -> i32 {
        if self.cfg.permadeath {
            RETREAT_PCT
        } else {
            OLD_RETREAT_PCT
        }
    }

    fn level_cap(&self) -> u32 {
        if self.won {
            self.cfg.postgame_level_cap
        } else {
            MAX_LEVEL
        }
    }

    fn pos(&self) -> (usize, usize) {
        (self.hero.x, self.hero.y)
    }

    fn npc_snaps(&self) -> Vec<NpcSnap> {
        let z = self.hero.zone;
        let mut v = Vec::new();
        if z == 8 {
            v.push(NpcSnap { id: "boss".to_string(), kind: "boss", species: BOSS, level: ZONES[8].max, x: self.boss.0, y: self.boss.1 });
        } else if z >= 1 {
            for (i, e) in self.elites.iter().enumerate() {
                if e.zone == z && e.dead_until == 0 {
                    v.push(NpcSnap { id: format!("e{}", i), kind: "elite", species: e.species, level: e.level, x: e.pos.0, y: e.pos.1 });
                }
            }
        }
        v
    }

    fn emit(&mut self, kind: &'static str, text: String, mon: Option<&Mon>) {
        if self.lean {
            self.emit_lean();
            return;
        }
        if self.done {
            return;
        }
        let h = &self.hero;
        let hero = HeroSnap {
            level: h.level,
            hp: h.hp.clamp(0, h.max_hp),
            max_hp: h.max_hp,
            xp: h.xp,
            gold: h.gold,
            potions: h.potions,
            zone: h.zone,
            weapon: h.weapon,
            armor: h.armor,
            x: h.x,
            y: h.y,
        };
        let mon = mon.map(|m| MonSnap {
            species: m.species,
            level: m.level,
            hp: m.hp.clamp(0, m.max_hp),
            max_hp: m.max_hp,
            stage: m.stage,
            x: m.x,
            y: m.y,
        });
        let npcs = self.npc_snaps();
        let path = std::mem::take(&mut self.next_path);
        let steps = std::mem::take(&mut self.leg_steps);
        let raw = std::mem::take(&mut self.leg_nsteps);
        let name = |id: u16| if id == BOSS_ID { "boss".to_string() } else { format!("e{}", id) };
        let nsteps = raw
            .iter()
            .map(|mv| mv.iter().map(|&(id, x, y)| (name(id), x, y)).filter(|m| npcs.iter().any(|n| n.id == m.0)).collect())
            .collect();
        let idx = self.count as u32;
        self.events.push(Event { idx, kind, text, hero, mon, npcs, path, steps, nsteps });
        self.count += 1;
        if self.count >= self.cfg.event_cap as usize {
            self.done = true;
            self.capped = true;
        }
    }

    fn target_zone(&self) -> usize {
        let lvl = self.hero.level;
        let margin = self.cfg.zone_margin;
        let mut best = 1;
        for z in 1..=7 {
            if z == 1 || ZONES[z].min + margin <= lvl {
                best = z;
            }
        }
        let gate = (!self.cfg.permadeath || self.boss_go) && lvl >= (self.cfg.boss_level + 2 * self.boss_deaths).min(MAX_LEVEL);
        if !self.won && gate {
            best = 8;
        }
        best
    }

    /// Emit the accumulated walk leg as one "walk" event.
    fn flush_walk(&mut self) {
        if self.leg.len() >= 2 {
            let tiles = self.leg_tiles;
            if self.lean {
                self.emit_lean();
            } else {
                self.next_path = self.leg.iter().map(|p| (p.0 as u16, p.1 as u16)).collect();
                let t = format!("Hero walks {} tiles to ({}, {})", tiles, self.hero.x, self.hero.y);
                self.emit("walk", t, None);
                self.next_path.clear();
            }
        }
        self.leg.clear();
        self.leg_steps.clear();
        self.leg_nsteps.clear();
        self.leg_tiles = 0;
    }

    fn announce_zone(&mut self, z: usize) {
        self.last_travel = z;
        let zn = &ZONES[z];
        ev!(self, "travel", None, "Hero travels to {} {}", zn.emoji, zn.name);
    }

    fn rest(&mut self) {
        if self.hero.hp < self.hero.max_hp {
            self.hero.hp = self.hero.max_hp;
            ev!(self, "rest", None, "Hero rests at Base Camp and recovers fully");
        }
    }

    fn buy_potions(&mut self, upto: u32) {
        let cost = self.cfg.potion_cost;
        while self.hero.potions < upto && self.hero.gold >= cost {
            self.hero.gold -= cost;
            self.hero.potions += 1;
            ev!(self, "shop", None, "Bought a potion for {} gold ({} held)", cost, self.hero.potions);
        }
    }

    fn shop(&mut self) {
        self.buy_potions(3);
        loop {
            let mut bought = false;
            let next_w = self.hero.weapon + 1;
            if next_w < WEAPONS.len() && self.hero.gold >= WEAPONS[next_w].1 {
                // buy best affordable weapon
                let mut b = next_w;
                while b + 1 < WEAPONS.len() && self.hero.gold >= WEAPONS[b + 1].1 {
                    b += 1;
                }
                self.hero.gold -= WEAPONS[b].1;
                self.hero.weapon = b;
                ev!(self, "shop", None, "Bought {} for {} gold", WEAPONS[b].0, WEAPONS[b].1);
                bought = true;
            }
            let next_a = self.hero.armor + 1;
            if next_a < ARMORS.len() && self.hero.gold >= ARMORS[next_a].1 {
                let mut b = next_a;
                while b + 1 < ARMORS.len() && self.hero.gold >= ARMORS[b + 1].1 {
                    b += 1;
                }
                self.hero.gold -= ARMORS[b].1;
                self.hero.armor = b;
                ev!(self, "shop", None, "Bought {} for {} gold", ARMORS[b].0, ARMORS[b].1);
                bought = true;
            }
            if !bought {
                break;
            }
        }
        self.buy_potions(5);
        if self.cfg.permadeath {
            self.buy_potions(9);
        }
    }

    fn gain_xp(&mut self, amount: u32) {
        let cap = self.level_cap();
        if self.hero.level >= cap {
            return;
        }
        self.hero.xp += amount;
        while self.hero.level < cap && self.hero.xp >= xp_to_next(self.hero.level) {
            self.hero.xp -= xp_to_next(self.hero.level);
            self.hero.level += 1;
            let new_max = Hero::max_hp_for(&self.cfg, self.hero.level);
            let gain = new_max - self.hero.max_hp;
            self.hero.max_hp = new_max;
            self.hero.hp = (self.hero.hp + gain).min(new_max);
            let lvl = self.hero.level;
            let mut text = String::new();
            if !self.lean {
                text = format!("Hero reaches level {}", lvl);
            }
            match lvl {
                3 => text.push_str(" and learns Power Strike"),
                6 => text.push_str(" and learns Brace"),
                10 => text.push_str(" and learns Stun"),
                14 => text.push_str(" and learns Heal"),
                _ => {}
            }
            self.emit("level_up", text, None);
        }
        if self.hero.level >= cap {
            self.hero.xp = 0;
        }
    }

    fn spawn(&mut self, species: usize, level: u32, stage: u8, elite: bool, loc: (usize, usize)) -> Mon {
        let s = &SPECIES[species];
        let mut hp = monster_stat(s.hp, level);
        let mut atk = monster_stat(s.atk, level);
        let mut def = monster_stat(s.def, level);
        if stage > 0 {
            let m = STAGE_MULT[stage as usize];
            hp *= m;
            atk *= m;
            def *= m;
        }
        hp *= self.cfg.monster_hp_mult;
        atk *= self.cfg.monster_atk_mult;
        if species == BOSS {
            hp *= self.cfg.boss_hp_mult;
            atk *= self.cfg.boss_atk_mult;
        }
        if elite {
            hp *= self.cfg.elite_mult;
            atk *= self.cfg.elite_mult;
        }
        let spd = monster_stat(s.spd, level);
        let mut packed = 1;
        if s.behavior == Behavior::Pack && !elite {
            packed = self.rng.range(2, 3) as u32;
            // weak copies bundled into one stronger foe
            hp *= 0.6 * packed as f32;
            atk *= 1.0 + 0.25 * (packed - 1) as f32;
        }
        let hp = (hp as i32).max(1);
        Mon { species, level, hp, max_hp: hp, atk, def, spd, behavior: s.behavior, packed, stage, x: loc.0, y: loc.1, elite }
    }

    fn fight(&mut self, species: usize, level: u32, stage: u8, loc: (usize, usize), elite_idx: Option<usize>) -> Outcome {
        let mut m = self.spawn(species, level, stage, elite_idx.is_some(), loc);
        let s = &SPECIES[species];
        let boss = species == BOSS;
        let kind = if boss {
            "boss"
        } else if m.elite {
            "elite"
        } else {
            "encounter"
        };
        if self.lean {
            self.emit_lean();
        } else {
            let sn = stage_name(species, stage as usize);
            let name = if m.packed > 1 {
                format!("pack of {} {} {}", m.packed, sn, s.emoji)
            } else {
                format!("{} {}", sn, s.emoji)
            };
            let t = format!("Hero meets a level {} {}{} ({})", m.level, if m.elite { "elite " } else { "" }, name, s.behavior.name());
            self.emit(kind, t, Some(&m));
        }

        // before the first blow: a hero that could be one-shot drinks up or walks away
        if self.cfg.permadeath {
            let worst = self.worst_hit(&m);
            let (hp, max) = (self.hero.hp, self.hero.max_hp);
            if hp <= worst {
                if self.hero.potions > 0 && (hp + max / 2).min(max) > worst {
                    self.hero.potions -= 1;
                    self.hero.hp = (hp + max / 2).min(max);
                    ev!(self, "potion", Some(&m), "Hero drinks a potion and heals {}", self.hero.hp - hp);
                } else if self.may_flee(boss) {
                    ev!(self, "rest", Some(&m), "Hero judges the {} too dangerous and backs away", s.name);
                    if let Some(i) = elite_idx {
                        self.elites[i].cooldown = self.steps + 25;
                    }
                    self.fight_cd = self.steps + FIGHT_COOLDOWN;
                    return Outcome::Fled;
                }
            }
        }

        let mut round = 0u32;
        let mut mon_actions = 0u32;
        let mut poison = 0u32;
        let mut braced = false;
        let mut stunned = false;
        let mut guard = false;
        let mut weak = 0u32;
        let (mut cd_power, mut cd_stun, mut cd_brace, mut cd_heal) = (0u32, 0u32, 0u32, 0u32);

        loop {
            if self.done {
                return Outcome::Fled;
            }
            round += 1;
            if round > MAX_ROUNDS {
                ev!(self, "rest", Some(&m), "Hero flees from the {}", s.name);
                if let Some(i) = elite_idx {
                    self.elites[i].cooldown = self.steps + 25;
                }
                self.fight_cd = self.steps + FIGHT_COOLDOWN;
                return Outcome::Fled;
            }
            cd_power = cd_power.saturating_sub(1);
            cd_stun = cd_stun.saturating_sub(1);
            cd_brace = cd_brace.saturating_sub(1);
            cd_heal = cd_heal.saturating_sub(1);

            let hero_first = self.hero.spd() >= m.spd;
            for step in 0..2 {
                let hero_turn = (step == 0) == hero_first;
                if hero_turn {
                    // --- hero turn ---
                    let lvl = self.hero.level;
                    let hp = self.hero.hp;
                    let max = self.hero.max_hp;
                    let charge_next = m.behavior == Behavior::Charger && mon_actions % 3 == 2;
                    let mut force_potion = false;
                    if self.cfg.permadeath {
                        let worst = self.worst_hit(&m);
                        let can_heal = self.hero.potions > 0 && (hp + max / 2).min(max) > worst;
                        let flee = if hp <= worst {
                            if can_heal {
                                force_potion = true;
                                false
                            } else {
                                true
                            }
                        } else {
                            self.too_risky(&m)
                        };
                        if flee && self.may_flee(boss) {
                            ev!(self, "rest", Some(&m), "Hero judges the {} too dangerous and backs away", s.name);
                            if let Some(i) = elite_idx {
                                self.elites[i].cooldown = self.steps + 25;
                            }
                            self.fight_cd = self.steps + FIGHT_COOLDOWN;
                            return Outcome::Fled;
                        }
                    }
                    let heal_at = if self.cfg.permadeath { HEAL_PCT } else { OLD_HEAL_PCT };
                    if (force_potion || hp * 100 < max * heal_at) && self.hero.potions > 0 {
                        self.hero.potions -= 1;
                        let heal = max / 2;
                        self.hero.hp = (hp + heal).min(max);
                        ev!(self, "potion", Some(&m), "Hero drinks a potion and heals {}", self.hero.hp - hp);
                    } else if lvl >= 14 && hp * 2 < max && cd_heal == 0 {
                        cd_heal = 5;
                        let heal = max * 3 / 10;
                        self.hero.hp = (hp + heal).min(max);
                        ev!(self, "skill", Some(&m), "Hero casts Heal and recovers {}", self.hero.hp - hp);
                    } else if lvl >= 6 && charge_next && cd_brace == 0 && !braced {
                        braced = true;
                        cd_brace = 3;
                        ev!(self, "skill", Some(&m), "Hero uses Brace to absorb the charge");
                    } else if lvl >= 10
                        && cd_stun == 0
                        && !stunned
                        && (m.behavior == Behavior::Charger || m.behavior == Behavior::Berserker)
                    {
                        cd_stun = 5;
                        stunned = true;
                        let d = damage(&mut self.rng, self.hero.atk(&self.cfg) * 0.5, m.def);
                        m.hp = (m.hp - d).max(0);
                        ev!(self, "skill", Some(&m), "Hero Stuns the {} for {}", s.name, d);
                    } else {
                        let (power, label) = if lvl >= 3 && cd_power == 0 {
                            cd_power = 3;
                            (true, "Power Strike")
                        } else {
                            (false, "attack")
                        };
                        let dodged = m.behavior == Behavior::Trickster && self.rng.chance(0.25);
                        if dodged {
                            ev!(self, "attack", Some(&m), "Hero's {} misses, the {} dodges", label, s.name);
                        } else {
                            let mut a = self.hero.atk(&self.cfg);
                            if power {
                                a *= 1.8;
                            }
                            if weak > 0 {
                                weak -= 1;
                                a *= 0.9;
                            }
                            let mut d = damage(&mut self.rng, a, m.def);
                            if m.behavior == Behavior::Defensive && round <= 2 {
                                d = (d / 2).max(1);
                            }
                            if guard {
                                guard = false;
                                d = (d / 2).max(1);
                            }
                            m.hp = (m.hp - d).max(0);
                            let kind = if power { "skill" } else { "attack" };
                            ev!(self, kind, Some(&m), "Hero uses {} on the {} for {}", label, s.name, d);
                        }
                    }
                } else {
                    // --- monster turn ---
                    if m.hp > 0 {
                        mon_actions += 1;
                        if stunned {
                            stunned = false;
                            ev!(self, "attack", Some(&m), "The {} is stunned and loses its turn", s.name);
                        } else if m.behavior == Behavior::Healer && m.hp < m.max_hp {
                            let heal = (m.max_hp / 10).max(1);
                            m.hp = (m.hp + heal).min(m.max_hp);
                            ev!(self, "attack", Some(&m), "The {} regenerates {}", s.name, heal);
                        } else {
                            let mv = pick_move(&mut self.rng, species);
                            let mut atk = m.atk * mv.power / mean_power(species);
                            let mut note = String::new();
                            match m.behavior {
                                Behavior::Aggressive => atk *= 1.25,
                                Behavior::Berserker if m.hp * 2 < m.max_hp => {
                                    atk *= 1.6;
                                    note = " in a frenzy".to_string();
                                }
                                Behavior::Charger if mon_actions % 3 == 0 => {
                                    atk *= 2.2;
                                    note = " with a mighty charge".to_string();
                                }
                                _ => {}
                            }
                            let mut d = damage(&mut self.rng, atk, self.hero.def(&self.cfg));
                            if braced && note.contains("charge") {
                                d = (d * 3 / 10).max(1);
                                braced = false;
                                note.push_str(" (braced)");
                            } else if braced && mon_actions % 3 != 0 {
                                // brace only lasts until the next hit lands
                                braced = false;
                            }
                            self.hero.hp = (self.hero.hp - d).max(0);
                            match mv.effect {
                                MoveEffect::Poison => {
                                    poison = 3;
                                    note.push_str(" and poisons the hero");
                                }
                                MoveEffect::Drain => {
                                    let h = (d / 3).max(1);
                                    m.hp = (m.hp + h).min(m.max_hp);
                                    note.push_str(&format!(" and drains {}", h));
                                }
                                MoveEffect::Guard => {
                                    guard = true;
                                    note.push_str(" and guards itself");
                                }
                                MoveEffect::Weaken => {
                                    weak = 1;
                                    note.push_str(" and weakens the hero");
                                }
                                MoveEffect::None => {
                                    if m.behavior == Behavior::Venomous && self.rng.chance(0.3) {
                                        poison = 3;
                                        note.push_str(" and poisons the hero");
                                    }
                                }
                            }
                            ev!(self, "hit_taken", Some(&m), "The {} uses {} on the hero{} for {}", s.name, mv.name, note, d);
                        }
                    }
                }
                if self.hero.hp <= 0 || m.hp <= 0 {
                    break;
                }
            }

            // end of round: poison
            if self.hero.hp > 0 && m.hp > 0 && poison > 0 {
                poison -= 1;
                let d = (self.hero.max_hp / 25).max(1);
                self.hero.hp = (self.hero.hp - d).max(0);
                ev!(self, "hit_taken", Some(&m), "Poison deals {} to the hero", d);
            }

            if self.hero.hp <= 0 {
                if self.won {
                    if !self.done {
                        self.post_deaths += 1;
                    }
                } else {
                    self.deaths += 1;
                }
                if boss {
                    self.boss_deaths += 1;
                    if self.boss_deaths >= self.cfg.boss_attempts {
                        self.done = true;
                    }
                }
                ev!(self, "death", Some(&m), "Hero is slain by the {}", s.name);
                if !self.won {
                    self.died_level = Some(self.hero.level);
                    self.death_zone = Some(self.hero.zone);
                }
                if self.cfg.permadeath {
                    ev!(self, "end", None, "The hero has fallen. The journey ends here");
                    self.ended = true;
                    self.done = true;
                    return Outcome::Died;
                }
                let lost = self.hero.gold / 4;
                self.hero.gold -= lost;
                let camp = self.map.camp();
                self.hero.x = camp.0;
                self.hero.y = camp.1;
                self.hero.zone = 0;
                self.last_travel = 0;
                self.hero.hp = self.hero.max_hp;
                self.hero.potions = self.hero.potions.min(5);
                ev!(self, "respawn", None, "Hero wakes at Base Camp, having lost {} gold", lost);
                return Outcome::Died;
            }
            if m.hp <= 0 {
                let mult = if m.elite { 2 } else { 1 };
                                let xp = ((s.xp * (0.5 + 0.25 * m.level as f32) * self.cfg.xp_mult) as u32 * m.packed.max(1) * mult).max(1);
                let gold = ((s.gold * m.level as f32 * 2.0 * self.cfg.gold_mult) as u32 * m.packed.max(1) * mult).max(1);
                self.hero.gold += gold;
                let mut text = String::new();
                if !self.lean {
                    text = format!("Hero defeats the {} and gains {} xp, {} gold", s.name, xp, gold);
                }
                if !boss && self.hero.potions < 9 && self.rng.chance(0.15) {
                    self.hero.potions += 1;
                    if !self.lean {
                        text.push_str(", finds a potion");
                    }
                }
                if self.won && !boss && !self.done {
                    self.post_kills += 1;
                }
                if let Some(i) = elite_idx {
                    let until = (self.steps + ELITE_RESPAWN_STEPS).max(1);
                    self.elites[i].dead_until = until;
                    self.next_respawn = self.next_respawn.min(until);
                }
                self.fight_cd = self.steps + FIGHT_COOLDOWN;
                self.emit("victory_fight", text, Some(&m));
                if boss {
                    if self.done {
                        return Outcome::Won;
                    }
                    self.won = true;
                    self.final_level = self.hero.level;
                    self.first_win_event = Some(self.count as u32);
                    ev!(self, "win", Some(&m), "The Jungle King falls. The jungle is saved!");
                    return Outcome::Won;
                }
                self.gain_xp(xp);
                return Outcome::Won;
            }
        }
    }

    fn check_evolve(&mut self) {
        if self.won && !self.done {
            let ns = ((self.post_kills / self.cfg.evolve_kills) as usize).min(MAX_STAGE) as u8;
            if ns > self.stage {
                self.stage = ns;
                ev!(self, "evolve", None, "The jungle stirs: monsters evolve to stage {}", ns);
            }
        }
    }

    /// Settle a finished fight into a walk result. `None` means the walk goes on.
    fn after_fight(&mut self, o: Outcome, boss: bool, check: u8) -> Option<Walk> {
        match o {
            Outcome::Died => Some(Walk::Died),
            Outcome::Fled => Some(Walk::Fled),
            Outcome::Won => {
                self.check_evolve();
                if self.done {
                    return Some(Walk::Aborted);
                }
                if boss {
                    return Some(Walk::Retreat);
                }
                let h = &self.hero;
                if check & CHECK_SUPPLIES != 0 && h.potions == 0 && h.hp * 100 < h.max_hp * self.retreat_pct() {
                    return Some(Walk::Retreat);
                }
                if check & CHECK_TARGET != 0 && self.target_zone() != self.goal {
                    return Some(Walk::Retarget);
                }
                None
            }
        }
    }

    fn boss_fight(&mut self, check: u8) -> Option<Walk> {
        self.flush_walk();
        let z = &ZONES[8];
        let level = self.rng.range(z.min as i32, z.max as i32) as u32;
        let loc = self.boss;
        self.boss_go = false;
        let o = self.fight(BOSS, level, 0, loc, None);
        self.after_fight(o, true, check)
    }

    fn respawn_elites(&mut self) {
        if self.steps < self.next_respawn {
            return;
        }
        let mut next = u32::MAX;
        for e in self.elites.iter_mut() {
            if e.dead_until != 0 {
                if self.steps >= e.dead_until {
                    e.dead_until = 0;
                    e.pos = e.home;
                } else {
                    next = next.min(e.dead_until);
                }
            }
        }
        self.next_respawn = next;
    }

    /// True when `p` keeps at least NPC_GAP from every other living NPC.
    fn npc_clear(&self, p: (usize, usize), me: Option<usize>) -> bool {
        if me.is_some() && dist(self.boss, p) < NPC_GAP {
            return false;
        }
        self.elites.iter().enumerate().all(|(j, e)| Some(j) == me || e.dead_until != 0 || dist(e.pos, p) >= NPC_GAP)
    }

    /// Elites and the boss only move while the hero is in their zone.
    fn step_npcs(&mut self) -> Vec<(u16, u16, u16)> {
        let mut moved = Vec::new();
        let hz = self.hero.zone;
        let hp = self.pos();
        if hz == 8 {
            if self.rng.chance(0.5) {
                let (nx, ny) = step_dir(&mut self.rng, self.boss);
                if nx >= 0 && ny >= 0 {
                    let np = (nx as usize, ny as usize);
                    let centre = self.map.anchors()[8];
                    let clear = self.elites.iter().all(|e| e.dead_until != 0 || dist(e.pos, np) >= NPC_GAP);
                    if self.map.passable(np.0, np.1) && self.map.zone_of(np.0, np.1) == 8 && dist(np, centre) <= 2 && np != hp && clear {
                        self.boss = np;
                        moved.push((BOSS_ID, np.0 as u16, np.1 as u16));
                    }
                }
            }
            return moved;
        }
        if hz == 0 {
            return moved;
        }
        for i in 0..self.elites.len() {
            if self.elites[i].zone != hz || self.elites[i].dead_until != 0 {
                continue;
            }
            if !self.rng.chance(0.5) {
                continue;
            }
            let pos = self.elites[i].pos;
            let (nx, ny) = if dist(pos, hp) <= ELITE_CHASE_RADIUS && pos != hp {
                let (dx, dy) = (hp.0 as i32 - pos.0 as i32, hp.1 as i32 - pos.1 as i32);
                let horiz = (pos.0 as i32 + dx.signum(), pos.1 as i32);
                let vert = (pos.0 as i32, pos.1 as i32 + dy.signum());
                if dx.abs() >= dy.abs() && dx != 0 {
                    horiz
                } else {
                    vert
                }
            } else {
                step_dir(&mut self.rng, pos)
            };
            if nx < 0 || ny < 0 {
                continue;
            }
            let np = (nx as usize, ny as usize);
            if np != hp
                && self.map.passable(np.0, np.1)
                && self.map.zone_of(np.0, np.1) == hz
                && dist(np, self.elites[i].home) <= ELITE_ROAM
                && self.npc_clear(np, Some(i))
            {
                self.elites[i].pos = np;
                moved.push((i as u16, np.0 as u16, np.1 as u16));
            }
        }
        moved
    }

    /// One tile of walking: movement, zone entry, NPC movement, contact fights, grass encounters.
    fn take_step(&mut self, p: (usize, usize), dest_zone: usize, check: u8) -> Option<Walk> {
        let prev = self.pos();
        self.hero.x = p.0;
        self.hero.y = p.1;
        let z = self.map.zone_of(p.0, p.1);
        self.hero.zone = z;
        self.steps += 1;
        let dir = (p.0 as i32 - prev.0 as i32, p.1 as i32 - prev.1 as i32);
        if self.leg.is_empty() {
            self.leg.push(prev);
            self.leg_dir = (0, 0);
            self.leg_tiles = 0;
        }
        if dir == self.leg_dir {
            let n = self.leg.len();
            self.leg[n - 1] = p;
        } else {
            self.leg.push(p);
            self.leg_dir = dir;
        }
        self.leg_tiles += 1;
        if !self.lean {
            self.leg_steps.push((p.0 as u16, p.1 as u16));
            self.leg_nsteps.push(Vec::new());
        }
        if self.steps >= self.cfg.max_steps {
            self.flush_walk();
            self.done = true;
            self.capped = true;
            return Some(Walk::Aborted);
        }
        // NPCs move before any flush so the event's snapshot includes this step's moves
        self.respawn_elites();
        let moved = self.step_npcs();
        if let Some(l) = self.leg_nsteps.last_mut() {
            *l = moved;
        }
        if self.leg.len() >= MAX_WAYPOINTS || self.leg_steps.len() >= MAX_LEG_STEPS {
            self.flush_walk();
        }
        if (z == dest_zone || z == 0) && z != self.last_travel {
            self.flush_walk();
            self.announce_zone(z);
            if self.done {
                return Some(Walk::Aborted);
            }
        }

        // contact fights (none while the post-fight cooldown runs)
        if self.steps >= self.fight_cd {
            if !self.won && z == 8 && self.target_zone() == 8 && dist(self.boss, p) <= 1 {
                return self.boss_fight(check);
            }
            if (1..=7).contains(&z) {
                for i in 0..self.elites.len() {
                    let e = &self.elites[i];
                    // the hero picks its fights: it steers clear of elites far above its level or when hurt
                    let brave = self.hero.level + ELITE_LEVEL_MARGIN >= e.level
                        && self.hero.hp * 2 >= self.hero.max_hp
                        && !(self.cfg.permadeath && self.won && self.stage >= 1);
                    let (hit, sp, lv, loc) = (
                        brave && e.zone == z && e.dead_until == 0 && e.cooldown <= self.steps && dist(e.pos, p) <= 1,
                        e.species,
                        e.level,
                        e.pos,
                    );
                    if hit {
                        self.flush_walk();
                        let stage = self.stage;
                        let o = self.fight(sp, lv, stage, loc, Some(i));
                        return self.after_fight(o, false, check);
                    }
                }
                if self.steps >= self.fight_cd && self.map.tile(p.0, p.1) == Tile::Grass && self.rng.chance(self.cfg.encounter_rate) {
                    self.flush_walk();
                    let zone = &ZONES[z];
                    let sp = pick_weighted(&mut self.rng, zone.species);
                    let lv = self.rng.range(zone.min as i32, zone.max as i32) as u32;
                    let stage = self.stage;
                    let o = self.fight(sp, lv, stage, p, None);
                    return self.after_fight(o, false, check);
                }
            }
        }
        if self.done {
            return Some(Walk::Aborted);
        }
        None
    }

    /// Shortest route `from` -> `to` (excluding `from`). Open L-shaped routes are already optimal,
    /// so they are taken directly; anything blocked falls back to A*.
    fn route(&self, from: (usize, usize), to: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        for horizontal_first in [true, false] {
            let mut out = Vec::with_capacity(dist(from, to));
            let (mut x, mut y) = from;
            let mut ok = self.map.passable(to.0, to.1);
            while ok && (x, y) != to {
                let move_x = if horizontal_first { x != to.0 } else { y == to.1 };
                if move_x {
                    x = if x < to.0 { x + 1 } else { x - 1 };
                } else {
                    y = if y < to.1 { y + 1 } else { y - 1 };
                }
                ok = self.map.passable(x, y);
                out.push((x, y));
            }
            if ok && self.map.passable(from.0, from.1) {
                return Some(out);
            }
        }
        self.map.path(from, to)
    }

    /// An L-shaped route (either bend) whose every tile is open and inside `zone`; no search.
    fn l_route_in_zone(&self, from: (usize, usize), to: (usize, usize), zone: usize) -> Option<Vec<(usize, usize)>> {
        'bend: for horizontal_first in [true, false] {
            let mut out = Vec::with_capacity(dist(from, to));
            let (mut x, mut y) = from;
            while (x, y) != to {
                let move_x = if horizontal_first { x != to.0 } else { y == to.1 };
                if move_x {
                    x = if x < to.0 { x + 1 } else { x - 1 };
                } else {
                    y = if y < to.1 { y + 1 } else { y - 1 };
                }
                if !self.map.passable(x, y) || self.map.zone_of(x, y) != zone {
                    continue 'bend;
                }
                out.push((x, y));
            }
            return Some(out);
        }
        None
    }

    fn walk_to(&mut self, dest: (usize, usize), dest_zone: usize, check: u8) -> Walk {
        let Some(path) = self.route(self.pos(), dest) else {
            self.done = true;
            return Walk::Aborted;
        };
        self.walk_route(path, dest_zone, check)
    }

    fn walk_route(&mut self, path: Vec<(usize, usize)>, dest_zone: usize, check: u8) -> Walk {
        for p in path {
            if let Some(w) = self.take_step(p, dest_zone, check) {
                return w;
            }
        }
        self.flush_walk();
        Walk::Arrived
    }

    fn go_home(&mut self) {
        let camp = self.map.camp();
        while !self.done && self.pos() != camp {
            if self.walk_to(camp, 0, 0) == Walk::Died {
                return;
            }
        }
    }

    fn pick_wander(&mut self, target: usize) -> (usize, usize) {
        let me = self.pos();
        let r = self.cfg.wander_radius as usize;
        let grass = &self.zone_grass[target];
        let rows = &self.zone_rows[target];
        // grass is row-major, so only the rows within the radius can hold a near tile
        let lo = rows[me.1.saturating_sub(r)];
        let hi = rows[(me.1 + r + 1).min(rows.len() - 1)];
        let near = grass[lo..hi].iter().filter(|&&p| p != me && dist(p, me) <= r).count();
        if near > 0 {
            let k = self.rng.range(0, near as i32 - 1) as usize;
            return self.zone_grass[target][lo..hi].iter().copied().filter(|&p| p != me && dist(p, me) <= r).nth(k).unwrap();
        }
        let n = grass.iter().filter(|&&p| p != me).count();
        if n == 0 {
            return me;
        }
        let k = self.rng.range(0, n as i32 - 1) as usize;
        self.zone_grass[target].iter().copied().filter(|&p| p != me).nth(k).unwrap()
    }

    fn chase_boss_step(&mut self) -> Walk {
        let me = self.pos();
        if dist(me, self.boss) <= 1 {
            return self.boss_fight(CHECK_SUPPLIES | CHECK_TARGET).unwrap_or(Walk::Arrived);
        }
        match self.map.path(me, self.boss) {
            Some(p) if !p.is_empty() => self.take_step(p[0], 8, CHECK_SUPPLIES | CHECK_TARGET).unwrap_or(Walk::Arrived),
            _ => Walk::Retreat,
        }
    }

    fn grind(&mut self, target: usize) {
        loop {
            if self.done {
                break;
            }
            let h = &self.hero;
            if h.potions == 0 && h.hp * 100 < h.max_hp * self.retreat_pct() {
                break;
            }
            if self.target_zone() != target {
                break;
            }
            let r = if target == 8 {
                self.chase_boss_step()
            } else {
                let mut wp = self.pick_wander(target);
                if wp == self.pos() {
                    break;
                }
                // stay inside the target zone: prefer a wander tile whose plain L route does,
                // and only fall back to a zone-bound A* when a few draws all leave the zone
                let mut direct = None;
                for attempt in 0..5 {
                    direct = self.l_route_in_zone(self.pos(), wp, target);
                    if direct.is_some() || attempt == 4 {
                        break;
                    }
                    wp = self.pick_wander(target);
                    if wp == self.pos() {
                        break;
                    }
                }
                if wp == self.pos() {
                    break;
                }
                match direct.or_else(|| self.map.path_in(self.pos(), wp, Some(target))) {
                    Some(path) => self.walk_route(path, target, CHECK_SUPPLIES | CHECK_TARGET),
                    None => self.walk_to(wp, target, CHECK_SUPPLIES | CHECK_TARGET),
                }
            };
            if r != Walk::Arrived {
                break;
            }
        }
        self.flush_walk();
    }

    fn run(&mut self) {
        let camp = self.map.camp();
        while !self.done {
            if self.pos() != camp {
                self.go_home();
                continue;
            }
            self.rest();
            self.shop();
            if self.done {
                break;
            }
            let h = &self.hero;
            let ready = h.hp * 10 >= h.max_hp * 8 && h.potions >= 5 && h.level >= self.boss_hold;
            self.boss_go = if !self.cfg.permadeath {
                true
            } else if !ready || self.won || self.hero.level < self.cfg.boss_level {
                false
            } else {
                // keep farming and shopping until the fight looks winnable, but never wait forever
                let since = *self.boss_wait.get_or_insert(self.steps);
                self.boss_favourable() || self.steps - since > BOSS_WAIT_STEPS
            };
            // outbound: re-aim in place when the target zone changes on the way
            let (target, outcome) = loop {
                let target = self.target_zone();
                self.goal = target;
                if self.outbound[target].is_none() {
                    let dest = self.map.nearest_tile_of_zone(camp, target).unwrap_or(camp);
                    let route = self.route(camp, dest).unwrap_or_default();
                    self.outbound[target] = Some((dest, route));
                }
                let (dest, cached) = self.outbound[target].clone().unwrap();
                let w = if self.pos() == camp { self.walk_route(cached, target, CHECK_SUPPLIES | CHECK_TARGET) } else { self.walk_to(dest, target, CHECK_SUPPLIES | CHECK_TARGET) };
                if w == Walk::Retarget || (w == Walk::Arrived && self.target_zone() != target) {
                    continue;
                }
                break (target, w);
            };
            if outcome == Walk::Arrived {
                self.grind(target);
            }
        }
    }
}

pub fn simulate(seed: u64) -> Run {
    simulate_with(seed, &Config::default())
}

fn run_sim(seed: u64, cfg: &Config, lean: bool) -> Sim {
    let max_hp = Hero::max_hp_for(cfg, 1);
    let map = Map::generate(seed, cfg.map_size as usize);
    let camp = map.camp();
    let boss = map.anchors()[8];
    let mut sim = Sim {
        cfg: cfg.clone(),
        rng: Rng::new(seed),
        hero: Hero { level: 1, hp: max_hp, max_hp, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0, x: camp.0, y: camp.1 },
        map,
        events: Vec::new(),
        deaths: 0,
        boss_deaths: 0,
        won: false,
        done: false,
        lean,
        count: 0,
        capped: false,
        died_level: None,
        death_zone: None,
        ended: false,
        boss_flees: 0,
        boss_hold: 0,
        flee_state: (0, 0, 0),
        outcome: "capped",
        boss_go: false,
        fight_cd: 0,
        first_win_event: None,
        final_level: 0,
        post_kills: 0,
        post_deaths: 0,
        stage: 0,
        steps: 0,
        elites: Vec::new(),
        next_respawn: u32::MAX,
        boss,
        goal: 1,
        last_travel: 0,
        leg: Vec::new(),
        leg_dir: (0, 0),
        leg_tiles: 0,
        next_path: Vec::new(),
        leg_steps: Vec::new(),
        leg_nsteps: Vec::new(),
        zone_grass: vec![Vec::new(); 9],
        zone_rows: vec![Vec::new(); 9],
        outbound: vec![None; 9],
    };
    let n = sim.map.size();
    for y in 0..n {
        for x in 0..n {
            if sim.map.tile(x, y) == Tile::Grass {
                let z = sim.map.zone_of(x, y);
                sim.zone_grass[z].push((x, y));
            }
        }
    }
    for z in 0..9 {
        let mut rows = vec![0usize; n + 1];
        for &(_, y) in &sim.zone_grass[z] {
            rows[y + 1] += 1;
        }
        for y in 0..n {
            rows[y + 1] += rows[y];
        }
        sim.zone_rows[z] = rows;
    }
    // roaming elites: species from the zone table, spaced home tiles inside the zone
    let homes = sim.map.elite_homes(seed, sim.cfg.elites_per_zone);
    let mut hi = 0;
    for z in 1..=7usize {
        for _ in 0..sim.cfg.elites_per_zone {
            let species = pick_weighted(&mut sim.rng, ZONES[z].species);
            let home = homes[hi];
            hi += 1;
            sim.elites.push(Elite { zone: z, species, level: ZONES[z].max + 2, home, pos: home, dead_until: 0, cooldown: 0 });
        }
    }
    ev!(sim, "travel", None, "Hero begins the journey at Base Camp");
    sim.run();
    if !sim.won {
        sim.final_level = sim.hero.level;
    }
    sim.outcome = if sim.won {
        "won"
    } else if sim.ended {
        "died"
    } else if sim.capped {
        "capped"
    } else {
        "died"
    };
    if sim.outcome != "died" {
        sim.died_level = None;
        sim.death_zone = None;
    }
    sim
}

pub fn simulate_with(seed: u64, cfg: &Config) -> Run {
    let sim = run_sim(seed, cfg, false);
    Run {
        seed,
        steps: sim.steps,
        map_size: cfg.map_size,
        final_level: sim.final_level,
        won: sim.won,
        deaths: sim.deaths,
        first_win_event: sim.first_win_event,
        end_level: sim.hero.level,
        post_kills: sim.post_kills,
        post_deaths: sim.post_deaths,
        max_stage: sim.stage,
        capped: sim.capped,
        outcome: sim.outcome,
        died_level: sim.died_level,
        death_zone: sim.death_zone,
        events: sim.events,
    }
}

pub fn simulate_summary(seed: u64, cfg: &Config) -> Summary {
    let sim = run_sim(seed, cfg, true);
    Summary {
        steps: sim.steps,
        won: sim.won,
        deaths: sim.deaths,
        final_level: sim.final_level,
        first_win_event: sim.first_win_event,
        events: sim.count as u32,
        end_level: sim.hero.level,
        post_kills: sim.post_kills,
        post_deaths: sim.post_deaths,
        max_stage: sim.stage,
        capped: sim.capped,
        outcome: sim.outcome,
        died_level: sim.died_level,
        death_zone: sim.death_zone,
    }
}
