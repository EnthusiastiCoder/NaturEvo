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
