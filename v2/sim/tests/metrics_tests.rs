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
