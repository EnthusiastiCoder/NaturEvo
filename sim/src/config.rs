/// Tunable simulation parameters. `Default` reproduces the original tuning exactly.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub monster_hp_mult: f32,
    pub monster_atk_mult: f32,
    pub boss_hp_mult: f32,
    pub boss_atk_mult: f32,
    pub hero_hp_base: i32,
    pub hero_hp_per_level: i32,
    pub hero_atk_per_level: f32,
    pub hero_def_per_level: f32,
    pub xp_mult: f32,
    pub gold_mult: f32,
    pub potion_cost: u32,
    pub boss_level: u32,
    pub boss_attempts: u32,
    pub event_cap: u32,
    pub postgame_level_cap: u32,
    pub evolve_kills: u32,
    pub map_size: u32,
    pub encounter_rate: f32,
    pub wander_radius: u32,
    pub elites_per_zone: u32,
    pub elite_mult: f32,
    pub max_steps: u32,
}

pub const MAX_EVENT_CAP: u32 = 1_048_576;

impl Default for Config {
    fn default() -> Self {
        Config {
            monster_hp_mult: 1.0,
            monster_atk_mult: 1.0,
            boss_hp_mult: 1.0,
            boss_atk_mult: 1.0,
            hero_hp_base: 50,
            hero_hp_per_level: 14,
            hero_atk_per_level: 3.0,
            hero_def_per_level: 1.5,
            xp_mult: 1.0,
            gold_mult: 1.0,
            potion_cost: 25,
            boss_level: 32,
            boss_attempts: 8,
            event_cap: 65536,
            postgame_level_cap: 80,
            evolve_kills: 300,
            map_size: 64,
            encounter_rate: 0.12,
            wander_radius: 6,
            elites_per_zone: 1,
            elite_mult: 1.4,
            max_steps: 5_000_000,
        }
    }
}

pub struct Knob {
    pub key: &'static str,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

static KNOBS: [Knob; 22] = [
    Knob { key: "monster_hp_mult", label: "Monster HP multiplier", min: 0.1, max: 5.0, default: 1.0 },
    Knob { key: "monster_atk_mult", label: "Monster attack multiplier", min: 0.1, max: 5.0, default: 1.0 },
    Knob { key: "boss_hp_mult", label: "Boss HP multiplier", min: 0.1, max: 5.0, default: 1.0 },
    Knob { key: "boss_atk_mult", label: "Boss attack multiplier", min: 0.1, max: 5.0, default: 1.0 },
    Knob { key: "hero_hp_base", label: "Hero base HP", min: 10.0, max: 500.0, default: 50.0 },
    Knob { key: "hero_hp_per_level", label: "Hero HP per level", min: 1.0, max: 60.0, default: 14.0 },
    Knob { key: "hero_atk_per_level", label: "Hero attack per level", min: 0.5, max: 12.0, default: 3.0 },
    Knob { key: "hero_def_per_level", label: "Hero defense per level", min: 0.0, max: 8.0, default: 1.5 },
    Knob { key: "xp_mult", label: "XP multiplier", min: 0.1, max: 10.0, default: 1.0 },
    Knob { key: "gold_mult", label: "Gold multiplier", min: 0.1, max: 10.0, default: 1.0 },
    Knob { key: "potion_cost", label: "Potion cost (gold)", min: 1.0, max: 500.0, default: 25.0 },
    Knob { key: "boss_level", label: "Level to challenge boss", min: 5.0, max: 40.0, default: 32.0 },
    Knob { key: "boss_attempts", label: "Boss attempts before loss", min: 1.0, max: 50.0, default: 8.0 },
    Knob { key: "event_cap", label: "Event cap (max events per run)", min: 100.0, max: 1048576.0, default: 65536.0 },
    Knob { key: "postgame_level_cap", label: "Post-game hero level cap", min: 40.0, max: 200.0, default: 80.0 },
    Knob { key: "evolve_kills", label: "Post-game kills per evolution", min: 10.0, max: 100000.0, default: 300.0 },
    Knob { key: "map_size", label: "Map size (tiles per side)", min: 32.0, max: 128.0, default: 64.0 },
    Knob { key: "encounter_rate", label: "Encounter chance per grass step", min: 0.01, max: 0.5, default: 0.12 },
    Knob { key: "wander_radius", label: "Hero wander radius (tiles)", min: 2.0, max: 20.0, default: 6.0 },
    Knob { key: "elites_per_zone", label: "Roaming elites per zone", min: 0.0, max: 3.0, default: 1.0 },
    Knob { key: "elite_mult", label: "Elite stat multiplier", min: 1.0, max: 3.0, default: 1.4 },
    Knob { key: "max_steps", label: "Step limit (safety cap)", min: 1000.0, max: 50000000.0, default: 5000000.0 },
];

pub fn knobs() -> &'static [Knob] {
    &KNOBS
}

fn knob(key: &str) -> Option<&'static Knob> {
    KNOBS.iter().find(|k| k.key == key)
}

fn check(k: &Knob, v: f64) -> Result<(), String> {
    // bounds are rounded to f32 so an f32 field set to its own bound (e.g. 0.01) passes
    if !v.is_finite() || v < k.min as f32 as f64 || v > k.max as f32 as f64 {
        return Err(format!("{}: value must be between {} and {}", k.key, k.min, k.max));
    }
    Ok(())
}

impl Config {
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let k = knob(key).ok_or_else(|| format!("unknown parameter '{}'", key))?;
        let bad = || format!("{}: invalid number '{}'", key, value);
        let v = value.trim();
        match key {
            "hero_hp_base" | "hero_hp_per_level" => {
                let n: i32 = v.parse().map_err(|_| bad())?;
                check(k, n as f64)?;
                if key == "hero_hp_base" {
                    self.hero_hp_base = n;
                } else {
                    self.hero_hp_per_level = n;
                }
            }
            "potion_cost" | "boss_level" | "boss_attempts" | "event_cap" | "postgame_level_cap" | "evolve_kills"
            | "map_size" | "wander_radius" | "elites_per_zone" | "max_steps" => {
                let n: u32 = v.parse().map_err(|_| bad())?;
                check(k, n as f64)?;
                match key {
                    "potion_cost" => self.potion_cost = n,
                    "boss_level" => self.boss_level = n,
                    "boss_attempts" => self.boss_attempts = n,
                    "event_cap" => self.event_cap = n,
                    "postgame_level_cap" => self.postgame_level_cap = n,
                    "map_size" => self.map_size = n,
                    "wander_radius" => self.wander_radius = n,
                    "elites_per_zone" => self.elites_per_zone = n,
                    "max_steps" => self.max_steps = n,
                    _ => self.evolve_kills = n,
                }
            }
            _ => {
                let n: f32 = v.parse().map_err(|_| bad())?;
                check(k, n as f64)?;
                match key {
                    "monster_hp_mult" => self.monster_hp_mult = n,
                    "monster_atk_mult" => self.monster_atk_mult = n,
                    "boss_hp_mult" => self.boss_hp_mult = n,
                    "boss_atk_mult" => self.boss_atk_mult = n,
                    "hero_atk_per_level" => self.hero_atk_per_level = n,
                    "hero_def_per_level" => self.hero_def_per_level = n,
                    "xp_mult" => self.xp_mult = n,
                    _ => self.gold_mult = n,
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        let vals: [(&str, f64); 16] = [
            ("monster_hp_mult", self.monster_hp_mult as f64),
            ("monster_atk_mult", self.monster_atk_mult as f64),
            ("boss_hp_mult", self.boss_hp_mult as f64),
            ("boss_atk_mult", self.boss_atk_mult as f64),
            ("hero_hp_base", self.hero_hp_base as f64),
            ("hero_hp_per_level", self.hero_hp_per_level as f64),
            ("hero_atk_per_level", self.hero_atk_per_level as f64),
            ("hero_def_per_level", self.hero_def_per_level as f64),
            ("xp_mult", self.xp_mult as f64),
            ("gold_mult", self.gold_mult as f64),
            ("potion_cost", self.potion_cost as f64),
            ("boss_level", self.boss_level as f64),
            ("boss_attempts", self.boss_attempts as f64),
            ("event_cap", self.event_cap as f64),
            ("postgame_level_cap", self.postgame_level_cap as f64),
            ("evolve_kills", self.evolve_kills as f64),
        ];
        for (key, v) in vals {
            check(knob(key).unwrap(), v)?;
        }
        Ok(())
    }
}
