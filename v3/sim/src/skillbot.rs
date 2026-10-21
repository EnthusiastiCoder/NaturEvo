//! Scripted opponents that fight through the same Executor as the net: three tiers for a curriculum.
//! Tier 1 Brawler (attacks only), tier 2 Reactive (reads the telegraph, guards or parries, punishes Open),
//! tier 3 Duellist (adds feints, drag, backstep, shield bash, riposte). Deterministic from the seed.
use crate::moveset::{MoveId, MoveParams, Phase};
use crate::rng::Rng;
use crate::robot::{Robot, State};
use crate::sense::MoveInfo;

pub type Request = Option<(MoveId, MoveParams)>;

const SLASHES: [MoveId; 6] = [MoveId::Overhead, MoveId::DiagDown, MoveId::Sweep, MoveId::Thrust, MoveId::DiagUp, MoveId::LowCut];
/// Torso distance band a bot swings from, metres.
const RANGE_LO: f32 = 1.9;
const RANGE_HI: f32 = 2.2;
const CLOSE: f32 = 1.35;
/// Retreat briefly after a passive opponent staggers, to avoid its recoil sweep.
const FLAIL_TIME: f32 = 1.0;
const FLAIL_DIST: f32 = 3.5;
const FLAIL_IDLE: f32 = 2.0;
const LOW_MOVE_P: f32 = 0.1;

pub fn is_attack(id: MoveId) -> bool {
    !matches!(id, MoveId::Idle | MoveId::Feint | MoveId::Guard | MoveId::Parry | MoveId::Backstep)
}

fn p(reach: f32, speed: f32, arc: f32, delay: f32) -> MoveParams {
    MoveParams { reach, speed, arc, delay }
}

pub struct SkillBot {
    tier: u8,
    rng: Rng,
    wait: f32,
    react: f32,
    reacted: bool,
    bait: Option<f32>,
    follow: bool,
    close_plan: f32,
    down: bool,
    was_stag: bool,
    flail_t: f32,
    opp_idle: f32,
}

impl SkillBot {
    pub fn new(seed: u64, tier: u8) -> SkillBot {
        let tier = tier.clamp(1, 3);
        let mut rng = Rng::new(seed ^ 0x51C1_1B07 ^ ((tier as u64) << 32));
        let wait = rng.range(0.1, 0.5);
        SkillBot { tier, rng, wait, react: -1.0, reacted: false, bait: None, follow: false, close_plan: 0.0, down: false, was_stag: false, flail_t: 0.0, opp_idle: 0.0 }
    }

    pub fn tier(&self) -> u8 {
        self.tier
    }

    fn attack_params(&mut self, id: MoveId) -> MoveParams {
        match self.tier {
            1 => p(self.rng.range(0.9, 1.0), self.rng.range(0.8, 1.0), self.rng.range(0.4, 0.5), 0.0),
            2 => p(self.rng.range(0.9, 1.0), self.rng.range(0.9, 1.0), self.rng.range(0.4, 0.5), 0.0),
            _ => {
                let delay = if id != MoveId::Thrust && self.rng.f32() < 0.25 { self.rng.range(0.4, 1.0) } else { 0.0 };
                p(self.rng.range(0.85, 1.0), self.rng.range(0.8, 1.0), self.rng.range(0.3, 0.6), delay)
            }
        }
    }
