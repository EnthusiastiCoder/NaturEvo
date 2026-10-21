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

    fn pick_attack(&mut self, dist: f32) -> MoveId {
        if dist < CLOSE && self.rng.f32() < 0.5 {
            return MoveId::Kick;
        }
        if self.rng.f32() < LOW_MOVE_P { return if self.rng.f32() < 0.5 { MoveId::DiagUp } else { MoveId::LowCut }; }
        SLASHES[(self.rng.next_u64() % 4) as usize]
    }

    /// One tick. Returns the move request (the executor ignores what it cannot start) and the footwork drive.
    pub fn act(&mut self, me: &Robot, opp: &Robot, mine: &MoveInfo, theirs: &MoveInfo, live_threat: bool, dt: f32) -> (Request, f32) {
        let dx = opp.x - me.x;
        let dist = dx.abs();
        let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
        let idle = mine.phase == Phase::Idle;
        self.wait = (self.wait - dt).max(0.0);

        // Footwork: close to the swing band; the smarter tiers also give ground when crowded.
        let down = opp.state() == State::Fallen;
        self.down = down;
        let lo = if self.close_plan > 0.0 || down { CLOSE } else { RANGE_LO };
        let mut drive = if dist > lo + 0.4 { dir } else { 0.0 };
        if self.tier >= 2 && !down && dist < RANGE_LO - 0.5 && self.close_plan <= 0.0 && idle {
            drive = -dir * 0.7;
        }
        self.close_plan = (self.close_plan - dt).max(0.0);
        let stag = opp.state() == State::Staggered;
        if stag && !self.was_stag { self.flail_t = FLAIL_TIME; }
        self.was_stag = stag;
        if theirs.phase != Phase::Idle { self.opp_idle = 0.0 } else { self.opp_idle += dt }
        self.flail_t = (self.flail_t - dt).max(0.0);
        let flail = stag && self.opp_idle > FLAIL_IDLE && self.flail_t > 0.0 && dist < FLAIL_DIST;
        if flail {
            drive = -dir;
        }

        // Feint bait in progress: cancel the windup, then strike for real at once.
        if let Some(at) = self.bait {
            if mine.phase == Phase::Windup && mine.progress >= at {
                self.bait = None;
                self.follow = true;
                self.wait = 0.0;
                return (Some((MoveId::Feint, p(0.5, 0.5, 0.5, 0.0))), drive);
            }
            if idle {
                self.bait = None;
            }
        }

        if self.tier >= 2 {
            let telegraph = is_attack(theirs.id) && matches!(theirs.phase, Phase::Windup | Phase::Active) || (live_threat && theirs.phase != Phase::Recovery);
            if telegraph {
                if !self.reacted {
                    self.reacted = true;
                    self.react = self.rng.range(0.1, 0.2);
                }
            } else {
                self.reacted = false;
                self.react = -1.0;
            }
            if self.react >= 0.0 {
                self.react -= dt;
                if self.react < 0.0 {
                    return (Some(self.defend(theirs)), drive);
                }
            }
            // Punish an Open opponent with the fastest in-range attack.
            if theirs.open && idle && dist <= RANGE_HI + 0.15 && dist >= RANGE_LO - 0.5 {
                let id = if self.rng.f32() < 0.5 { MoveId::DiagUp } else { MoveId::Thrust };
                return (Some((id, p(0.9, 1.0, 0.5, 0.0))), drive);
            }
        }
        if self.tier == 3 && mine.riposte_remaining > 0.0 && idle && dist <= RANGE_HI + 0.3 {
            return (Some((MoveId::Riposte, p(0.9, 1.0, 0.5, 0.0))), drive);
        }

        let in_range = dist <= if down { 1.85 } else { RANGE_HI + 0.15 };
        if self.follow && idle {
            self.follow = false;
            if in_range {
                let id = self.pick_attack(dist);
                let pr = self.attack_params(id);
                return (Some((id, pr)), drive);
            }
        }
        if idle && self.wait <= 0.0 && self.react < 0.0 && !(flail) {
            if self.tier == 3 && theirs.id == MoveId::Guard && theirs.phase == Phase::Active && self.rng.f32() < 0.6 {
                // Shield bash from close in.
                if dist <= CLOSE + 0.3 {
                    self.wait = self.rng.range(0.3, 0.7);
                    return (Some((MoveId::ShieldBash, p(0.8, 0.9, 0.5, 0.0))), drive);
                }
                self.close_plan = 0.6;
            }
            if in_range && (dist >= RANGE_LO - 0.5 || down) {
                let id = self.pick_attack(dist);
                let pr = self.attack_params(id);
                let (w0, w1) = match self.tier {
                    1 => (0.1, 0.45),
                    2 => (0.7, 1.5),
                    _ => (0.4, 1.1),
                };
                self.wait = self.rng.range(w0, w1);
                if self.tier == 3 && id != MoveId::Kick && self.rng.f32() < 0.3 {
                    self.bait = Some(self.rng.range(0.3, 0.55));
                }
                return (Some((id, pr)), drive);
            }
        }
        (None, drive)
    }
