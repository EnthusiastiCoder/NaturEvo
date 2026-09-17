#![allow(dead_code)]
// Shared helpers for combat-depth tests. See v2/tests-index-combat-depth.md for the assumed hooks.
use arena_sim::attacks::{Rank, Species};
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::{Match, Setup};
use arena_sim::net::{N_OUT, N_PARAMS};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

pub fn idle() -> Action { Action::idle() }
pub fn atk(kind: u8) -> Action { Action { attack: true, attack_kind: kind, ..Action::idle() } }
pub fn blk() -> Action { Action { block: true, ..Action::idle() } }
pub fn dge() -> Action { Action { dodge: true, ..Action::idle() } }
pub fn fwd(v: f32) -> Action { Action { fwd: v, ..Action::idle() } }

/// Hero vs Minion grass, `dist` apart in the open centre band, facing each other.
pub fn duel(dist: f32) -> Match { duel_sp(Species::Minion, Rank::Grass, dist) }

pub fn duel_sp(monster: Species, rank: Rank, dist: f32) -> Match {
    duel_both(Species::Hero, monster, rank, dist)
}

pub fn duel_both(hero: Species, monster: Species, rank: Rank, dist: f32) -> Match {
    let cfg = ArenaConfig::default();
    let seed = 1u64;
    let mut r = Rng::new(seed);
    let h = Genome::random(Role::Hero, &mut r);
    let m = Genome::random(Role::Monster, &mut r);
    let setup = Setup { hp: [cfg.max_hp, cfg.max_hp], species: [hero, monster], rank: [Rank::Grass, rank] };
    let mut mt = Match::new_with_setup(Arena::generate(seed, &cfg), h, m, seed, &cfg, setup);
    mt.place_for_duel(dist);
    mt
}

pub fn run(m: &mut Match, n: u32, a: Action, b: Action) {
    for _ in 0..n { m.step_with(a, b); }
}

/// Genome whose net outputs are constant biases (all weights zero): `bias[i]` sets output i.
pub fn bias_genome(role: Role, bias: &[(usize, f32)]) -> Genome {
    let mut w = vec![0.0f32; N_PARAMS];
    let b2 = N_PARAMS - N_OUT;
    for (i, v) in bias { w[b2 + i] = *v; }
    Genome { role, weights: w, generation: 0, fitness: 0.0 }
}
/// Always blocks.
pub fn turtle(role: Role) -> Genome { bias_genome(role, &[(4, 1.0), (3, -1.0)]) }
/// Walks forward and attacks (light).
pub fn aggressor(role: Role) -> Genome { bias_genome(role, &[(0, 1.0), (3, 1.0), (4, -1.0)]) }
/// Does nothing.
pub fn idler(role: Role) -> Genome { bias_genome(role, &[(3, -1.0), (4, -1.0), (5, -1.0)]) }

pub fn dist(m: &Match) -> f32 {
    let (a, b) = (m.pos(Role::Hero), m.pos(Role::Monster));
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}
