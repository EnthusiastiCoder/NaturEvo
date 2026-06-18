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
