//! Combat. WS2 owns the implementation; WS1 only declares the event types the shared test helpers name.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClashKind {
    Deflect,
    Clash,
    PerfectParry,
}

#[derive(Clone, Debug)]
pub struct HitEvent {}

#[derive(Clone, Debug)]
pub enum CombatEvent {
    Hit(HitEvent),
    Clash { kind: ClashKind },
}
