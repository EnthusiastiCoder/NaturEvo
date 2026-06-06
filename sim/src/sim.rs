use crate::data::*;
use crate::rng::Rng;

const EVENT_CAP: usize = 300_000;
const MAX_LEVEL: u32 = 40;
const MAX_ROUNDS: u32 = 60;
const BOSS_LEVEL: u32 = 32;

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
}

#[derive(Clone, Debug)]
pub struct MonSnap {
    pub species: usize,
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub idx: u32,
    pub kind: &'static str,
    pub text: String,
    pub hero: HeroSnap,
    pub mon: Option<MonSnap>,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub seed: u64,
    pub events: Vec<Event>,
    pub won: bool,
    pub deaths: u32,
    pub final_level: u32,
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
}

impl Hero {
    fn max_hp_for(level: u32) -> i32 {
        50 + 14 * level as i32
    }
    fn atk(&self) -> f32 {
        8.0 + 3.0 * self.level as f32 + WEAPONS[self.weapon].2 as f32
    }
    fn def(&self) -> f32 {
        3.0 + 1.5 * self.level as f32 + ARMORS[self.armor].2 as f32
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
}

struct Sim {
    rng: Rng,
    hero: Hero,
    events: Vec<Event>,
    deaths: u32,
    boss_deaths: u32,
    won: bool,
    done: bool,
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
    fn emit(&mut self, kind: &'static str, text: String, mon: Option<&Mon>) {
        if self.done {
            return;
        }
        if self.events.len() >= EVENT_CAP {
            self.done = true;
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
        });
        let idx = self.events.len() as u32;
        self.events.push(Event { idx, kind, text, hero, mon });
        if self.events.len() >= EVENT_CAP {
            self.done = true;
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
        if lvl >= (BOSS_LEVEL + 2 * self.boss_deaths).min(MAX_LEVEL) {
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
        self.emit("travel", format!("Hero travels to {} {}", z.emoji, z.name), None);
    }

    fn rest(&mut self) {
        if self.hero.hp < self.hero.max_hp {
            self.hero.hp = self.hero.max_hp;
            self.emit("rest", "Hero rests at Base Camp and recovers fully".to_string(), None);
        }
    }

    fn buy_potions(&mut self, upto: u32) {
        while self.hero.potions < upto && self.hero.gold >= POTION_COST {
            self.hero.gold -= POTION_COST;
            self.hero.potions += 1;
            self.emit(
                "shop",
                format!("Bought a potion for {} gold ({} held)", POTION_COST, self.hero.potions),
                None,
            );
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
                self.emit("shop", format!("Bought {} for {} gold", WEAPONS[b].0, WEAPONS[b].1), None);
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
                self.emit("shop", format!("Bought {} for {} gold", ARMORS[b].0, ARMORS[b].1), None);
                bought = true;
            }
            if !bought {
                break;
            }
        }
        self.buy_potions(5);
    }

    fn gain_xp(&mut self, amount: u32) {
        if self.hero.level >= MAX_LEVEL {
            return;
        }
        self.hero.xp += amount;
        while self.hero.level < MAX_LEVEL && self.hero.xp >= xp_to_next(self.hero.level) {
            self.hero.xp -= xp_to_next(self.hero.level);
            self.hero.level += 1;
            let new_max = Hero::max_hp_for(self.hero.level);
            let gain = new_max - self.hero.max_hp;
            self.hero.max_hp = new_max;
            self.hero.hp = (self.hero.hp + gain).min(new_max);
            let lvl = self.hero.level;
            let mut text = format!("Hero reaches level {}", lvl);
            match lvl {
                3 => text.push_str(" and learns Power Strike"),
                6 => text.push_str(" and learns Brace"),
                10 => text.push_str(" and learns Stun"),
                14 => text.push_str(" and learns Heal"),
                _ => {}
            }
            self.emit("level_up", text, None);
        }
        if self.hero.level >= MAX_LEVEL {
            self.hero.xp = 0;
        }
    }

    fn spawn(&mut self, species: usize, level: u32) -> Mon {
        let s = &SPECIES[species];
        let mut hp = monster_stat(s.hp, level);
        let mut atk = monster_stat(s.atk, level);
        let def = monster_stat(s.def, level);
        let spd = monster_stat(s.spd, level);
        let mut packed = 1;
        if s.behavior == Behavior::Pack {
            packed = self.rng.range(2, 3) as u32;
            // weak copies bundled into one stronger foe
            hp *= 0.6 * packed as f32;
            atk *= 1.0 + 0.25 * (packed - 1) as f32;
        }
        let hp = (hp as i32).max(1);
        Mon { species, level, hp, max_hp: hp, atk, def, spd, behavior: s.behavior, packed }
    }

    fn fight(&mut self, species: usize, level: u32) -> Outcome {
        let mut m = self.spawn(species, level);
        let s = &SPECIES[species];
        let boss = species == BOSS;
        let name = if m.packed > 1 {
            format!("pack of {} {} {}", m.packed, s.name, s.emoji)
        } else {
            format!("{} {}", s.name, s.emoji)
        };
        let kind = if boss { "boss" } else { "encounter" };
        self.emit(
            kind,
            format!("Hero meets a level {} {} ({})", m.level, name, s.behavior.name()),
            Some(&m),
        );

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
                self.emit("rest", format!("Hero flees from the {}", s.name), Some(&m));
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
                        self.emit("potion", format!("Hero drinks a potion and heals {}", self.hero.hp - hp), Some(&m));
                    } else if lvl >= 14 && hp * 2 < max && cd_heal == 0 {
                        cd_heal = 5;
                        let heal = max * 3 / 10;
                        self.hero.hp = (hp + heal).min(max);
                        self.emit("skill", format!("Hero casts Heal and recovers {}", self.hero.hp - hp), Some(&m));
                    } else if lvl >= 6 && charge_next && cd_brace == 0 && !braced {
                        braced = true;
                        cd_brace = 3;
                        self.emit("skill", "Hero uses Brace to absorb the charge".to_string(), Some(&m));
                    } else if lvl >= 10
                        && cd_stun == 0
                        && !stunned
                        && (m.behavior == Behavior::Charger || m.behavior == Behavior::Berserker)
                    {
                        cd_stun = 5;
                        stunned = true;
                        let d = damage(&mut self.rng, self.hero.atk() * 0.5, m.def);
                        m.hp = (m.hp - d).max(0);
                        self.emit("skill", format!("Hero Stuns the {} for {}", s.name, d), Some(&m));
                    } else {
                        let (power, label) = if lvl >= 3 && cd_power == 0 {
                            cd_power = 3;
                            (true, "Power Strike")
                        } else {
                            (false, "attack")
                        };
                        let dodged = m.behavior == Behavior::Trickster && self.rng.chance(0.25);
                        if dodged {
                            self.emit("attack", format!("Hero's {} misses, the {} dodges", label, s.name), Some(&m));
                        } else {
                            let mut a = self.hero.atk();
                            if power {
                                a *= 1.8;
                            }
                            let mut d = damage(&mut self.rng, a, m.def);
                            if m.behavior == Behavior::Defensive && round <= 2 {
                                d = (d / 2).max(1);
                            }
                            m.hp = (m.hp - d).max(0);
                            let kind = if power { "skill" } else { "attack" };
                            self.emit(kind, format!("Hero uses {} on the {} for {}", label, s.name, d), Some(&m));
                        }
                    }
                } else {
                    // --- monster turn ---
                    if m.hp > 0 {
                        mon_actions += 1;
                        if stunned {
                            stunned = false;
                            self.emit("attack", format!("The {} is stunned and loses its turn", s.name), Some(&m));
                        } else if m.behavior == Behavior::Healer && m.hp < m.max_hp {
                            let heal = (m.max_hp / 10).max(1);
                            m.hp = (m.hp + heal).min(m.max_hp);
                            self.emit("attack", format!("The {} regenerates {}", s.name, heal), Some(&m));
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
                            let mut d = damage(&mut self.rng, atk, self.hero.def());
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
                            self.emit("hit_taken", format!("The {} hits the hero{} for {}", s.name, note, d), Some(&m));
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
                self.emit("hit_taken", format!("Poison deals {} to the hero", d), Some(&m));
            }

            if self.hero.hp <= 0 {
                self.deaths += 1;
                if boss {
                    self.boss_deaths += 1;
                }
                self.emit("death", format!("Hero is slain by the {}", s.name), Some(&m));
                let lost = self.hero.gold / 4;
                self.hero.gold -= lost;
                self.hero.zone = 0;
                self.hero.hp = self.hero.max_hp;
                self.hero.potions = self.hero.potions.min(5);
                self.emit(
                    "respawn",
                    format!("Hero wakes at Base Camp, having lost {} gold", lost),
                    None,
                );
                return Outcome::Died;
            }
            if m.hp <= 0 {
                let xp = ((s.xp * (0.5 + 0.25 * m.level as f32)) as u32 * m.packed.max(1)).max(1);
                let gold = ((s.gold * m.level as f32 * 2.0) as u32 * m.packed.max(1)).max(1);
                self.hero.gold += gold;
                let mut text = format!("Hero defeats the {} and gains {} xp, {} gold", s.name, xp, gold);
                if !boss && self.hero.potions < 9 && self.rng.chance(0.15) {
                    self.hero.potions += 1;
                    text.push_str(", finds a potion");
                }
                self.emit("victory_fight", text, Some(&m));
                if boss {
                    self.won = true;
                    self.emit("win", "The Jungle King falls. The jungle is saved!".to_string(), Some(&m));
                    return Outcome::Won;
                }
                self.gain_xp(xp);
                return Outcome::Won;
            }
        }
    }

    fn run(&mut self) {
        while !self.done && !self.won {
            // at camp
            self.travel(0);
            self.rest();
            self.shop();
            let target = self.target_zone();
            self.travel(target);
            loop {
                if self.done || self.won {
                    break;
                }
                let z = &ZONES[self.hero.zone];
                let species = if self.hero.zone == 8 {
                    BOSS
                } else {
                    z.species[self.rng.range(0, z.species.len() as i32 - 1) as usize]
                };
                let level = self.rng.range(z.min as i32, z.max as i32) as u32;
                if self.hero.zone == 8 {
                    // boss fights at full health
                }
                match self.fight(species, level) {
                    Outcome::Died => break,
                    Outcome::Fled => {
                        self.travel(0);
                        break;
                    }
                    Outcome::Won => {}
                }
                if self.won || self.done {
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
    let max_hp = Hero::max_hp_for(1);
    let mut sim = Sim {
        rng: Rng::new(seed),
        hero: Hero { level: 1, hp: max_hp, max_hp, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 },
        events: Vec::new(),
        deaths: 0,
        boss_deaths: 0,
        won: false,
        done: false,
    };
    sim.emit("travel", "Hero begins the journey at Base Camp".to_string(), None);
    sim.run();
    Run { seed, final_level: sim.hero.level, won: sim.won, deaths: sim.deaths, events: sim.events }
}
