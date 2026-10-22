// Downward-thrust finisher (plunge): Overhead and Thrust reach a fallen opponent when the executor knows the
// target is down (Executor::set_target_down). Tests first: see plans/reports for the CTO decision.
use mech_sim::combat::{resolve_tick_with, CombatEvent, Mods};
use mech_sim::executor::Executor;
use mech_sim::moveset::*;
use mech_sim::robot::{Joint, Part, Role, Robot, DT};
use mech_sim::rules;

mod common;
use common::*;

fn fast() -> MoveParams { MoveParams { reach: 1.0, speed: 1.0, arc: 0.5, delay: 0.0 } }

fn fallen_at(x: f32) -> Robot {
    let mut d = standing(Role::Monster, x, -1.0);
    d.knock_down();
    d.fk();
    d
}

/// Run `id` from a standing attacker at x=0 against a fallen defender; returns (hit parts, damage, plunge flag seen).
fn run(id: MoveId, att: &mut Robot, d: &mut Robot, down_flag: bool) -> Vec<(Part, f32)> {
    let mut ex = Executor::new();
    let ar = arena();
    let mut out = Vec::new();
    for i in 0..160 {
        ex.set_target_down(down_flag);
        let a = ex.step(att, if i == 0 { Some((id, fast())) } else { None }, DT);
        att.apply_targets(&a.targets, DT);
        for ev in resolve_tick_with(att, d, &ar, &Mods::default()) {
            if let CombatEvent::Hit(h) = ev {
                out.push((h.part, h.damage));
            }
        }
    }
    out
}
