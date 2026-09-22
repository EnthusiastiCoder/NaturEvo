// WS4 metrics. API (see v2/tests-index-ws46.md): metrics::analyze(&FightLog, Role, Outcome) -> FightMetrics
// { cells: [[[f32;6];4];2] indexed [seen][dist bucket][action], statuses_applied: [u32;7], punish_hits: u32,
//   parries: u32, attack_kind_share: [f32;3], draw: bool, decisive: bool }, FightMetrics::block_unseen_share() -> f32.
// Dist buckets: 0 close<=1.2, 1 mid<=4, 2 far<=12, 3 vfar. Actions: 0 ATTACK 1 BLOCK 2 DODGE 3 MOVE 4 IDLE 5 INVOLUNTARY.
// Action classes follow the fighter's intent bits; INVOLUNTARY = ticks in Staggered/Stunned/Knocked.
mod common;
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::Role;
use arena_sim::matchup::{play_with_setup, MatchResult, Setup};
use arena_sim::metrics::analyze;
use arena_sim::world::Arena;
use common::*;

fn fight(h: arena_sim::genome::Genome, m: arena_sim::genome::Genome, seed: u64) -> MatchResult {
    let cfg = ArenaConfig::default();
    let setup = Setup { hp: [cfg.max_hp, cfg.max_hp], species: [Species::Hero, Species::Minion], rank: [Rank::Grass, Rank::Grass] };
    play_with_setup(Arena::generate(seed, &cfg), h, m, seed, &cfg, setup)
}

#[test]
fn cells_sum_to_one_per_fighter() {
    let r = fight(aggressor(Role::Hero), aggressor(Role::Monster), 3);
    for role in [Role::Hero, Role::Monster] {
        let m = analyze(&r.log, role, r.outcome);
        let total: f32 = m.cells.iter().flatten().flatten().sum();
        assert!((total - 1.0).abs() < 1e-4, "{:?} total {total}", role);
        assert!(m.cells.iter().flatten().flatten().all(|v| *v >= 0.0));
    }
}

#[test]
fn idle_pair_is_all_unseen_vfar_idle_and_a_draw() {
    let r = fight(idler(Role::Hero), idler(Role::Monster), 1);
    let m = analyze(&r.log, Role::Hero, r.outcome);
    assert!(m.draw && !m.decisive);
    assert!((m.cells[0][3][4] - 1.0).abs() < 1e-4, "idle/unseen/vfar share {}", m.cells[0][3][4]);
    assert_eq!(m.block_unseen_share(), 0.0);
    assert_eq!(m.statuses_applied, [0u32; 7]);
    assert_eq!(m.punish_hits, 0);
    assert_eq!(m.parries, 0);
}

#[test]
fn turtle_blocking_unseen_is_counted_as_unseen_block() {
    let r = fight(turtle(Role::Hero), idler(Role::Monster), 1);
    let m = analyze(&r.log, Role::Hero, r.outcome);
    assert!(m.cells[0][3][1] > 0.8, "block/unseen/vfar {}", m.cells[0][3][1]);
    assert!(m.block_unseen_share() > 0.8, "{}", m.block_unseen_share());
    assert!(m.cells[1].iter().flatten().sum::<f32>() < 1e-4, "no seen ticks expected at 13 tiles");
}

#[test]
fn aggressor_attacks_seen_close_and_applies_stagger() {
    let r = fight(aggressor(Role::Hero), idler(Role::Monster), 2);
    let m = analyze(&r.log, Role::Hero, r.outcome);
    let seen_attack: f32 = m.cells[1].iter().map(|d| d[0]).sum();
    assert!(seen_attack > 0.0, "no seen attack ticks");
    assert!(m.cells[1][0][0] > 0.0, "no close-range attack ticks");
    assert!(m.statuses_applied[1] >= 1, "Light attack staggers (index 1): {:?}", m.statuses_applied);
    assert!((m.attack_kind_share[0] - 1.0).abs() < 1e-4, "{:?}", m.attack_kind_share);
    assert_eq!(m.attack_kind_share[1], 0.0);
    assert_eq!(m.attack_kind_share[2], 0.0);
    assert!(m.decisive && !m.draw);
    assert!(m.block_unseen_share() < 0.05);
    assert!(m.punish_hits >= 1, "idle target in recovery or staggered is punished");
}
