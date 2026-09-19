//! Fighter types: roles, actions, combat state.
use crate::status::StatusState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Hero,
    Monster,
}

impl Role {
    pub fn index(self) -> usize {
        match self {
            Role::Hero => 0,
            Role::Monster => 1,
        }
    }
    pub fn other(self) -> Role {
        match self {
            Role::Hero => Role::Monster,
            Role::Monster => Role::Hero,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Hero => "hero",
            Role::Monster => "monster",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub fwd: f32,
    pub strafe: f32,
    pub turn: f32,
    pub attack: bool,
    pub block: bool,
    pub dodge: bool,
    /// Which move of the fighter's moveset to use when attacking (wraps by moveset length).
    pub attack_kind: u8,
}

impl Action {
    pub fn idle() -> Action {
        Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false, attack_kind: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatState {
    Idle,
    Windup,
    Active,
    Recovery,
    Blocking,
    Dodging,
    Dead,
}

impl CombatState {
    pub fn as_str(self) -> &'static str {
        match self {
            CombatState::Idle => "Idle",
            CombatState::Windup => "Windup",
            CombatState::Active => "Active",
            CombatState::Recovery => "Recovery",
            CombatState::Blocking => "Blocking",
            CombatState::Dodging => "Dodging",
            CombatState::Dead => "Dead",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub x: f32,
    pub y: f32,
    pub facing: f32,
    pub hp: f32,
    pub stamina: f32,
    pub state: CombatState,
    pub timer: u32,
    pub dodge_cooldown: u32,
    pub last_action: Action,
}

impl Fighter {
    pub fn new(x: f32, y: f32, facing: f32, hp: f32, stamina: f32) -> Fighter {
        Fighter {
            x,
            y,
            facing,
            hp,
            stamina,
            state: CombatState::Idle,
            timer: 0,
            dodge_cooldown: 0,
            last_action: Action::idle(),
        }
    }
}
