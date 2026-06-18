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
