//! Posture: the ready stance is a natural upright fighting stance and fights do not hold a heavy lean.
use mech_sim::duel::{Brain, Match};
use mech_sim::robot::{Joint, Part, Robot, Role, State, DT, STAND_TARGETS};

/// Forward torso tilt in degrees from the capsule geometry (what the viewer draws); positive = toward facing.
fn torso_tilt(r: &Robot) -> f32 {
    let c = r.capsule(Part::Torso);
    ((c.b[0] - c.a[0]) * r.facing).atan2(c.b[1] - c.a[1]).to_degrees()
}
