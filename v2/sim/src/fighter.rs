//! Fighter types: roles, actions, combat state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Hero,
    Monster,
}
