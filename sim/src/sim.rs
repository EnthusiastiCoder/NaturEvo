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
const ELITE_CHASE_RADIUS: usize = 3;
const ELITE_LEVEL_MARGIN: u32 = 3;
const MAX_WAYPOINTS: usize = 64;

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
    /// grass tiles per zone, row-major
    zone_grass: Vec<Vec<(usize, usize)>>,
    /// cached camp -> zone entry tile and route, per zone
    outbound: Vec<Option<((usize, usize), Vec<(usize, usize)>)>>,
}

enum Outcome {
    Won,
    Died,
    Fled,
}

fn damage(rng: &mut Rng, atk: f32, def: f32) -> i32 {
    let base = (atk - def * 0.5).max(atk * 0.1).max(1.0);
    let v = base * (0.85 + 0.3 * rng.f());
    (v as i32).max(1)
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

    fn level_cap(&self) -> u32 {
        if self.won {
            self.cfg.postgame_level_cap
        } else {
            MAX_LEVEL
        }
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
        };
        let mon = mon.map(|m| MonSnap {
            species: m.species,
            level: m.level,
            hp: m.hp.clamp(0, m.max_hp),
            max_hp: m.max_hp,
            stage: m.stage,
        });
        let idx = self.count as u32;
        self.events.push(Event { idx, kind, text, hero, mon });
        self.count += 1;
        if self.count >= self.cfg.event_cap as usize {
            self.done = true;
            self.capped = true;
        }
    }

    fn target_zone(&self) -> usize {
        let lvl = self.hero.level;
        let mut best = 1;
        for z in 1..=7 {
            if ZONES[z].min <= lvl {
                best = z;
            }
        }
        if !self.won && lvl >= (self.cfg.boss_level + 2 * self.boss_deaths).min(MAX_LEVEL) {
            best = 8;
        }
        best
    }

    fn travel(&mut self, to: usize) {
        if self.hero.zone == to {
            return;
        }
        self.hero.zone = to;
        let z = &ZONES[to];
        ev!(self, "travel", None, "Hero travels to {} {}", z.emoji, z.name);
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

    fn spawn(&mut self, species: usize, level: u32, stage: u8) -> Mon {
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
        let spd = monster_stat(s.spd, level);
        let mut packed = 1;
        if s.behavior == Behavior::Pack {
            packed = self.rng.range(2, 3) as u32;
            // weak copies bundled into one stronger foe
            hp *= 0.6 * packed as f32;
            atk *= 1.0 + 0.25 * (packed - 1) as f32;
        }
        let hp = (hp as i32).max(1);
        Mon { species, level, hp, max_hp: hp, atk, def, spd, behavior: s.behavior, packed, stage }
    }

    fn fight(&mut self, species: usize, level: u32, stage: u8) -> Outcome {
        let mut m = self.spawn(species, level, stage);
        let s = &SPECIES[species];
        let boss = species == BOSS;
        let kind = if boss { "boss" } else { "encounter" };
        if self.lean {
            self.emit_lean();
        } else {
            let sn = stage_name(species, stage as usize);
            let name = if m.packed > 1 {
                format!("pack of {} {} {}", m.packed, sn, s.emoji)
            } else {
                format!("{} {}", sn, s.emoji)
            };
            let t = format!("Hero meets a level {} {} ({})", m.level, name, s.behavior.name());
            self.emit(kind, t, Some(&m));
        }

        let mut round = 0u32;
        let mut mon_actions = 0u32;
        let mut poison = 0u32;
        let mut braced = false;
        let mut stunned = false;
        let (mut cd_power, mut cd_stun, mut cd_brace, mut cd_heal) = (0u32, 0u32, 0u32, 0u32);

        loop {
            if self.done {
                return Outcome::Fled;
            }
            round += 1;
            if round > MAX_ROUNDS {
                ev!(self, "rest", Some(&m), "Hero flees from the {}", s.name);
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
                    if hp * 100 < max * 35 && self.hero.potions > 0 {
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
                            let mut d = damage(&mut self.rng, a, m.def);
                            if m.behavior == Behavior::Defensive && round <= 2 {
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
                            let mut atk = m.atk;
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
                            if m.behavior == Behavior::Venomous && self.rng.chance(0.6) {
                                poison = 3;
                                note.push_str(" and poisons the hero");
                            }
                            ev!(self, "hit_taken", Some(&m), "The {} hits the hero{} for {}", s.name, note, d);
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
                let lost = self.hero.gold / 4;
                self.hero.gold -= lost;
                self.hero.zone = 0;
                self.hero.hp = self.hero.max_hp;
                self.hero.potions = self.hero.potions.min(5);
                ev!(self, "respawn", None, "Hero wakes at Base Camp, having lost {} gold", lost);
                return Outcome::Died;
            }
            if m.hp <= 0 {
                let xp = ((s.xp * (0.5 + 0.25 * m.level as f32) * self.cfg.xp_mult) as u32 * m.packed.max(1)).max(1);
                let gold = ((s.gold * m.level as f32 * 2.0 * self.cfg.gold_mult) as u32 * m.packed.max(1)).max(1);
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

    fn run(&mut self) {
        while !self.done {
            // at camp
            self.travel(0);
            self.rest();
            self.shop();
            let target = self.target_zone();
            self.travel(target);
            loop {
                if self.done {
                    break;
                }
                if self.won {
                    let ns = ((self.post_kills / self.cfg.evolve_kills) as usize).min(MAX_STAGE) as u8;
                    if ns > self.stage {
                        self.stage = ns;
                        ev!(self, "evolve", None, "The jungle stirs: monsters evolve to stage {}", ns);
                        if self.done {
                            break;
                        }
                    }
                }
                let z = &ZONES[self.hero.zone];
                let species = if self.hero.zone == 8 {
                    BOSS
                } else {
                    z.species[self.rng.range(0, z.species.len() as i32 - 1) as usize].0
                };
                let level = self.rng.range(z.min as i32, z.max as i32) as u32;
                if self.hero.zone == 8 {
                    // boss fights at full health
                }
                let stage = if species == BOSS { 0 } else { self.stage };
                match self.fight(species, level, stage) {
                    Outcome::Died => break,
                    Outcome::Fled => {
                        self.travel(0);
                        break;
                    }
                    Outcome::Won => {}
                }
                if self.done {
                    break;
                }
                let h = &self.hero;
                if h.potions == 0 && h.hp * 2 < h.max_hp {
                    self.travel(0);
                    break;
                }
                if self.target_zone() != self.hero.zone {
                    self.travel(0);
                    break;
                }
            }
        }
    }
}

pub fn simulate(seed: u64) -> Run {
    simulate_with(seed, &Config::default())
}

fn run_sim(seed: u64, cfg: &Config, lean: bool) -> Sim {
    let max_hp = Hero::max_hp_for(cfg, 1);
    let mut sim = Sim {
        cfg: cfg.clone(),
        rng: Rng::new(seed),
        hero: Hero { level: 1, hp: max_hp, max_hp, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 },
        events: Vec::new(),
        deaths: 0,
        boss_deaths: 0,
        won: false,
        done: false,
        lean,
        count: 0,
        capped: false,
        first_win_event: None,
        final_level: 0,
        post_kills: 0,
        post_deaths: 0,
        stage: 0,
    };
    ev!(sim, "travel", None, "Hero begins the journey at Base Camp");
    sim.run();
    if !sim.won {
        sim.final_level = sim.hero.level;
    }
    sim
}

pub fn simulate_with(seed: u64, cfg: &Config) -> Run {
    let sim = run_sim(seed, cfg, false);
    Run {
        seed,
        final_level: sim.final_level,
        won: sim.won,
        deaths: sim.deaths,
        first_win_event: sim.first_win_event,
        end_level: sim.hero.level,
        post_kills: sim.post_kills,
        post_deaths: sim.post_deaths,
        max_stage: sim.stage,
        capped: sim.capped,
        events: sim.events,
    }
}

pub fn simulate_summary(seed: u64, cfg: &Config) -> Summary {
    let sim = run_sim(seed, cfg, true);
    Summary {
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
    }
}
