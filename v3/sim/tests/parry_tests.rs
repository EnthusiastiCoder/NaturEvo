// W2 sword-vs-sword resolution: deflect, clash, perfect parry. Geometry: attacker blade horizontal sweeping
// downward (3.4 m -> 2.8 m) across a vertical defender blade at x=3.0.
mod common;
use common::{arena, clashes, duel_pair, hits, seg};
use mech_sim::combat::{resolve_tick, ClashKind};
use mech_sim::robot::Robot;

fn approx(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{} vs {}", a, b);
}

/// vy: defender blade vertical velocity in m/s (positive = up, toward the descending attacker blade).
fn setup(vy: f32, moving_since: f32) -> (Robot, Robot) {
    let (mut a, mut d) = duel_pair();
    a.set_sword(seg(1.8, 3.4, 4.0, 3.4, 0.05), seg(1.8, 2.8, 4.0, 2.8, 0.05));
    a.sword_ang_vel = 10.0;
    a.strike_quality = 1.0;
    a.posture = 0.0;
    let dy = vy / 60.0;
    d.set_sword(seg(3.0, 1.5 - dy, 3.0, 3.7 - dy, 0.05), seg(3.0, 1.5, 3.0, 3.7, 0.05));
    d.sword_ang_vel = 8.0;
    d.sword_moving_since = moving_since;
    d.posture = 0.0;
    (a, d)
}
