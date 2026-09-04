//! Fighter types: roles, actions, combat state.

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
}

impl Action {
    pub fn idle() -> Action {
        Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false }
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
