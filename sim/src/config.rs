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
}

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

static KNOBS: [Knob; 13] = [
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
];

pub fn knobs() -> &'static [Knob] {
    &KNOBS
}

fn knob(key: &str) -> Option<&'static Knob> {
    KNOBS.iter().find(|k| k.key == key)
}

fn check(k: &Knob, v: f64) -> Result<(), String> {
    if !v.is_finite() || v < k.min || v > k.max {
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
            "potion_cost" | "boss_level" | "boss_attempts" => {
                let n: u32 = v.parse().map_err(|_| bad())?;
                check(k, n as f64)?;
                match key {
                    "potion_cost" => self.potion_cost = n,
                    "boss_level" => self.boss_level = n,
                    _ => self.boss_attempts = n,
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
