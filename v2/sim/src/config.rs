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

impl Default for CombatTuning {
    fn default() -> Self {
        CombatTuning {
            block_half_arc: BLOCK_HALF_ARC,
            block_reduction: 0.8,
            block_absorb_cost: 15.0,
            block_hold_drain: 6.0,
            guard_raise_ticks: 6,
            guard_recover_ticks: 8,
            guard_break_stun: 20,
            parry_window: 5,
            parry_stagger: 14,
            blocked_stun_stagger: 8,
            dodge_stamina_cost: 30.0,
            dodge_ticks: 8,
            dodge_cooldown: 45,
            stamina_regen_per_sec: 12.0,
            whiff_recovery_mult: 1.5,
            punish_mult: 1.25,
            stagger_immunity: 12,
            stagger_turn_mult: 0.5,
            stun_immunity: 45,
            stun_dr_window: 300,
            stun_dr_floor: 6,
            knock_ticks: 6,
            knock_immunity: 10,
            knock_block_mult: 0.5,
            wall_slam_damage: 3.0,
            wall_slam_stagger: 10,
            wall_slam_min_short: 0.3,
            confused_window: 15,
            confused_angle: 60.0 * std::f32::consts::PI / 180.0,
            slowed_max_stacks: 2,
            slowed_factor: 0.6,
            slowed_floor: 0.36,
            slowed_turn_mult: 0.7,
            bleed_max_stacks: 3,
            bleed_move_threshold: 0.5,
            bleed_move_mult: 2.0,
            dot_interval: 15,
            dot_damage: 1.0,
            poison_cap: 300,
            poison_regen_mult: 0.5,
            phase_move_factor: 0.3,
            guard_move_factor: 0.5,
            dodge_move_factor: 1.5,
        }
    }
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
    pub combat: CombatTuning,
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
            combat: CombatTuning::default(),
        }
    }
}
