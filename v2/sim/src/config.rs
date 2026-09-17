//! Arena configuration (world units are tiles).

/// Block half arc: 90 degrees, strictly narrower than the vision half-FOV.
pub const BLOCK_HALF_ARC: f32 = std::f32::consts::PI / 4.0;

/// All combat constants (ticks at 30 Hz). Move tables live in `attacks.rs`.
#[derive(Clone, Copy, Debug)]
pub struct CombatTuning {
    pub block_half_arc: f32,
    pub block_reduction: f32,
    pub block_absorb_cost: f32,
    pub block_hold_drain: f32,
    pub guard_raise_ticks: u32,
    pub guard_recover_ticks: u32,
    pub guard_break_stun: u32,
    pub parry_window: u32,
    pub parry_stagger: u32,
    pub blocked_stun_stagger: u32,
    pub dodge_stamina_cost: f32,
    pub dodge_ticks: u32,
    pub dodge_cooldown: u32,
    pub stamina_regen_per_sec: f32,
    pub whiff_recovery_mult: f32,
    pub punish_mult: f32,
    pub stagger_immunity: u32,
    pub stagger_turn_mult: f32,
    pub stun_immunity: u32,
    pub stun_dr_window: u32,
    pub stun_dr_floor: u32,
    pub knock_ticks: u32,
    pub knock_immunity: u32,
    pub knock_block_mult: f32,
    pub wall_slam_damage: f32,
    pub wall_slam_stagger: u32,
    pub wall_slam_min_short: f32,
    pub confused_window: u32,
    pub confused_angle: f32,
    pub slowed_max_stacks: u8,
    pub slowed_factor: f32,
    pub slowed_floor: f32,
    pub slowed_turn_mult: f32,
    pub bleed_max_stacks: u8,
    pub bleed_move_threshold: f32,
    pub bleed_move_mult: f32,
    pub dot_interval: u32,
    pub dot_damage: f32,
    pub poison_cap: u32,
    pub poison_regen_mult: f32,
    pub phase_move_factor: f32,
    pub guard_move_factor: f32,
    pub dodge_move_factor: f32,
}
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
