//! A single real-time 1v1 duel: physics, combat state machine, logging.
use crate::config::ArenaConfig;
use crate::fighter::{Action, CombatState, Fighter, Role};
use crate::genome::Genome;
use crate::log::{FightLog, Frame};
use crate::net::Net;
use crate::sense::observe;
use crate::world::Arena;
use std::f32::consts::PI;

pub const WINDUP_TICKS: u32 = 6;
pub const ACTIVE_TICKS: u32 = 4;
pub const RECOVERY_TICKS: u32 = 8;
pub const ATTACK_RANGE: f32 = 1.2;
pub const ATTACK_HALF_ARC: f32 = PI / 4.0; // 90 degree arc
pub const DAMAGE: f32 = 12.0;
pub const BLOCK_HALF_ARC: f32 = PI / 3.0; // 120 degree arc
pub const BLOCK_REDUCTION: f32 = 0.8;
pub const BLOCK_STAMINA_COST: f32 = 15.0;
pub const DODGE_STAMINA_COST: f32 = 30.0;
pub const DODGE_TICKS: u32 = 8;
pub const DODGE_COOLDOWN: u32 = 45;
pub const STAMINA_REGEN_PER_SEC: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    HeroWin,
    MonsterWin,
    Draw,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub outcome: Outcome,
    pub ticks: u32,
    pub hero_hp: f32,
    pub monster_hp: f32,
    pub damage_dealt: [f32; 2],
    pub log: FightLog,
}

#[derive(Clone, Debug)]
pub struct Match {
    cfg: ArenaConfig,
    arena: Arena,
    fighters: [Fighter; 2],
    nets: [Net; 2],
    tick: u32,
    finished: bool,
    outcome: Outcome,
    damage_dealt: [f32; 2],
    log: FightLog,
    #[allow(dead_code)]
    seed: u64,
}

fn wrap_angle(a: f32) -> f32 {
    let mut a = (a + PI).rem_euclid(2.0 * PI) - PI;
    if a <= -PI {
        a += 2.0 * PI;
    }
    a
}

fn clean(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

impl Match {
    pub fn new(arena: Arena, hero: Genome, monster: Genome, seed: u64, cfg: &ArenaConfig) -> Match {
        let s = arena.spawns();
        let fighters = [
            Fighter::new(s[0].0, s[0].1, 0.0, cfg.max_hp, cfg.max_stamina),
            Fighter::new(s[1].0, s[1].1, PI, cfg.max_hp, cfg.max_stamina),
        ];
        Match {
            cfg: cfg.clone(),
            arena,
            fighters,
            nets: [Net::from_genome(&hero), Net::from_genome(&monster)],
            tick: 0,
            finished: false,
            outcome: Outcome::Draw,
            damage_dealt: [0.0; 2],
            log: FightLog::new(cfg.tick_hz),
            seed,
        }
    }
