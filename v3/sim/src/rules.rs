//! Game-rule lethality for the sword vocabulary: Open windows, riposte, guard break, heavy block, backstep counter.
//! Pure data and functions: no RNG, no fitness or net access.
use crate::moveset::MoveId;
use crate::robot::Robot;

pub const OPEN_MULT: f32 = 1.30;
pub const OPEN_POSTURE_MULT: f32 = 1.5;
pub const COUNTER_HIT_MULT: f32 = 1.15;
pub const MULT_CAP: f32 = 1.75;
pub const RIPOSTE_WINDOW: f32 = 0.6;
pub const RIPOSTE_MULT: f32 = 1.5;
pub const RIPOSTE_WINDUP_FAST: f32 = 0.15;
pub const RIPOSTE_WINDUP_SLOW: f32 = 0.30;
pub const PARRY_MISS_RECOVERY: f32 = 0.30;
pub const PARRY_MISS_OPEN_MULT: f32 = 1.2;
pub const SHIELD_BASH_POSTURE: f32 = 35.0;
pub const GUARD_BREAK_TIME: f32 = 0.4;
pub const GUARD_BREAK_OPEN_MULT: f32 = 1.25;
pub const HEAVY_BLOCK_POSTURE_MULT: f32 = 1.4;
pub const BACKSTEP_COUNTER_POSTURE: f32 = 30.0;
pub const BACKSTEP_COUNTER_OPEN: f32 = 0.3;
pub const FEINT_COST: f32 = 8.0;
pub const DRAG_COST: f32 = 3.0;
pub const CHAIN_COST: f32 = 5.0;
/// Multiplier on exposed (staggered) victims.
const EXPOSED_MULT: f32 = 1.5;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RuleState {
    open_t: f32,
    open_m: f32,
    riposte_t: f32,
    guard_broken_t: f32,
}

impl RuleState {
    pub fn new() -> RuleState {
        RuleState::default()
    }
    pub fn tick(&mut self, dt: f32) {
        self.open_t = (self.open_t - dt).max(0.0);
        self.riposte_t = (self.riposte_t - dt).max(0.0);
        self.guard_broken_t = (self.guard_broken_t - dt).max(0.0);
    }
    fn open_for(&mut self, t: f32, m: f32) {
        self.open_t = t;
        self.open_m = m;
    }
    pub fn begin_recovery(&mut self, whiffed: bool, recovery: f32) {
        if whiffed {
            self.open_for(recovery, OPEN_MULT);
        }
    }
    pub fn on_parry_success(&mut self) {
        self.riposte_t = RIPOSTE_WINDOW;
    }
    pub fn on_parry_miss(&mut self) {
        self.open_for(PARRY_MISS_RECOVERY, PARRY_MISS_OPEN_MULT);
    }
    pub fn is_open(&self) -> bool {
        self.open_t > 0.0
    }
    pub fn open_remaining(&self) -> f32 {
        self.open_t
    }
    pub fn open_mult(&self) -> f32 {
        if self.is_open() { self.open_m } else { 1.0 }
    }
    pub fn riposte_open(&self) -> bool {
        self.riposte_t > 0.0
    }
    pub fn riposte_remaining(&self) -> f32 {
        self.riposte_t
    }
    pub fn guard_broken_remaining(&self) -> f32 {
        self.guard_broken_t
    }
}

/// Best multiplier (never a product), at least 1.0, capped at MULT_CAP.
pub fn combine(mults: &[f32]) -> f32 {
    mults.iter().fold(1.0f32, |a, &m| a.max(m)).min(MULT_CAP)
}

/// Damage multiplier on a hit; `st` is the victim's state.
pub fn punish_mult(st: &RuleState, victim_in_windup_or_active: bool, victim_staggered: bool, riposte_hit: bool) -> f32 {
    let mut v = vec![st.open_mult()];
    if victim_in_windup_or_active {
        v.push(COUNTER_HIT_MULT);
    }
    if victim_staggered {
        v.push(EXPOSED_MULT);
    }
    if riposte_hit && (victim_staggered || st.is_open()) {
        v.push(RIPOSTE_MULT);
    }
    combine(&v)
}

pub fn riposte_windup(st: &RuleState) -> f32 {
    if st.riposte_open() { RIPOSTE_WINDUP_FAST } else { RIPOSTE_WINDUP_SLOW }
}

/// ShieldBash landed on a guarding or blocking defender.
pub fn guard_break(defender: &mut Robot, def_st: &mut RuleState) {
    defender.add_posture(SHIELD_BASH_POSTURE);
    def_st.guard_broken_t = GUARD_BREAK_TIME;
    def_st.open_for(GUARD_BREAK_TIME, GUARD_BREAK_OPEN_MULT);
}

pub fn heavy_block_posture(m: MoveId, base_posture: f32) -> f32 {
    match m {
        MoveId::Overhead | MoveId::Sweep => base_posture * HEAVY_BLOCK_POSTURE_MULT,
        _ => base_posture,
    }
}
