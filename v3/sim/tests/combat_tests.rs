// W2 combat: swept collision, hit location and direction, knockback, hit-stop, block, caps.
mod common;
use common::{arena, blade_sweep, clashes, duel_pair, hits, seg, torso_center};
use mech_sim::combat::{hitstop_ticks, resolve_tick, swept_hit, CombatEvent};
use mech_sim::config::IMPACT_CAP;
use mech_sim::robot::Part;

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}
