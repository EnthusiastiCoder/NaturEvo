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
