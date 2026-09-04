//! Arena configuration (world units are tiles).

#[derive(Clone, Debug)]
pub struct ArenaConfig {
    pub tick_hz: u32,
    pub arena_w: f32,
    pub arena_h: f32,
    pub fighter_radius: f32,
    pub max_speed: f32,
    pub max_turn: f32,
    pub max_ticks: u32,
    pub n_rays: usize,
    pub vision_fov: f32,
    pub vision_range: f32,
    pub max_stamina: f32,
    pub max_hp: f32,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        ArenaConfig {
            tick_hz: 30,
            arena_w: 48.0,
            arena_h: 32.0,
            fighter_radius: 0.4,
            max_speed: 4.0,
            max_turn: 6.0,
            max_ticks: 1800,
            n_rays: 9,
            vision_fov: 2.094,
            vision_range: 12.0,
            max_stamina: 100.0,
            max_hp: 100.0,
        }
    }
}
