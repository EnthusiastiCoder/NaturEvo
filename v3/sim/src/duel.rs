//! Duel loop: two robots, brains, combat per tick, threat tracking, anti-stalemate bleed, outcome.
use crate::arena::Arena;
use crate::combat::{resolve_tick, CombatEvent, ClashKind};
use crate::config::{DECISION_TICKS, IMPACT_REF, MAX_TICKS, PHYS_DT};
use crate::genome::Genome;
use crate::log::{self, f2, FightLog};
use crate::moves::raw_impact;
use crate::net::Net;
use crate::robot::{Capsule, Joint, Part, Robot, Role, State, STAND_TARGETS};
use crate::scripted::{self, Action, Scripted};
use crate::sense::observe;

/// Who controls a fighter.
#[derive(Clone, Debug)]
pub enum Brain {
    Net(Genome),
    /// Stands still.
    Dummy,
    /// Approaches and swings at random intervals.
    Scripted,
    /// Holds the given normalised joint targets.
    Fixed([f32; 12]),
}

enum Ctl {
    Net(Box<Net>, Action),
    Dummy,
    Scripted(Box<Scripted>),
    Fixed(Action),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    WinA,
    WinB,
    Draw,
}

/// How a live threat ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Hit,
    Parried,
    Blocked,
    Dodged,
    Miss,
}

#[derive(Clone, Debug)]
pub struct Threat {
    pub id: u32,
    pub attacker: usize,
    pub defender: usize,
    pub start_tick: u32,
    pub resolution: Option<Resolution>,
    /// Impact the swing could have delivered (from its peak tip speed).
    pub potential: f32,
    start_x: f32,
    start_angles: [f32; 12],
    moved: f32,
    flexed: f32,
    quiet: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DefenceTally {
    pub parry: u32,
    pub block: u32,
    pub dodge: u32,
    pub hit: u32,
    pub miss: u32,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub outcome: Outcome,
    pub ticks: u32,
    pub defence: [DefenceTally; 2],
    pub idle_guard_share: [f32; 2],
    /// Raw sum of per-threat credits (each at most 1.0, at most 6 threats) earned by each defender.
    pub defence_credit: [f32; 2],
}

const THREAT_SPEED: f32 = 2.5;
const THREAT_RANGE: f32 = 0.25;
const THREAT_HORIZON: f32 = 0.4;
const SWING_COOLDOWN: u32 = 12;
const IDLE_GUARD_WINDOW: u32 = 36;
const TURN_TICKS: u32 = 15;
const BLEED_START: u32 = 450;
const BLEED_PER_SEC: f32 = 0.5;
const SPAWN_A: f32 = 4.5;
const SPAWN_B: f32 = 9.5;

fn seg_point_dist(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let l2 = d[0] * d[0] + d[1] * d[1];
    let t = if l2 < 1e-9 { 0.0 } else { (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / l2).clamp(0.0, 1.0) };
    let c = [a[0] + d[0] * t, a[1] + d[1] * t];
    ((c[0] - p[0]).powi(2) + (c[1] - p[1]).powi(2)).sqrt()
}

/// Minimum distance between two segments (sampled on the first, which is short).
fn seg_seg_dist(a: [f32; 2], b: [f32; 2], c: &Capsule) -> f32 {
    let mut best = f32::MAX;
    for i in 0..=8 {
        let t = i as f32 / 8.0;
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        best = best.min(seg_point_dist(c.a, c.b, p) - c.r);
    }
    best
}

/// A blade tip at `tip` moving at `vel` is a live threat to `def`: faster than 2.5 m/s and its
/// predicted path comes within 0.4 m of a defender body capsule within 0.4 s.
pub fn is_live_threat(tip: [f32; 2], vel: [f32; 2], def: &Robot) -> bool {
    let speed = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
    if speed <= THREAT_SPEED || !speed.is_finite() {
        return false;
    }
    let end = [tip[0] + vel[0] * THREAT_HORIZON, tip[1] + vel[1] * THREAT_HORIZON];
    Part::ALL.iter().any(|p| {
        if def.is_detached(*p) {
            return false;
        }
        let c = def.capsule(*p);
        let mid = [(c.a[0] + c.b[0]) * 0.5 - tip[0], (c.a[1] + c.b[1]) * 0.5 - tip[1]];
        mid[0] * vel[0] + mid[1] * vel[1] > 0.0 && seg_seg_dist(tip, end, &c) <= THREAT_RANGE
    })
}

pub struct Match {
    pub robots: [Robot; 2],
    pub arena: Arena,
    pub tick: u32,
    pub threats: Vec<Threat>,
    pub idle_guard_ticks: [u32; 2],
    pub log: Option<FightLog>,
    ctl: [Ctl; 2],
    hitstop: u32,
    cooldown: [u32; 2],
    turn: [u32; 2],
    last_live: [Option<u32>; 2],
    prev_state: [State; 2],
    prev_destroyed: [[bool; 13]; 2],
    next_threat: u32,
    ended: bool,
}

fn make_ctl(b: Brain, seed: u64, salt: u64) -> Ctl {
    match b {
        Brain::Net(g) => Ctl::Net(Box::new(Net::from_genome(&g)), Action { targets: STAND_TARGETS, drive: 0.0 }),
        Brain::Dummy => Ctl::Dummy,
        Brain::Scripted => Ctl::Scripted(Box::new(Scripted::new(seed.wrapping_add(salt)))),
        Brain::Fixed(t) => Ctl::Fixed(Action { targets: t, drive: 0.0 }),
    }
}

impl Match {
    pub fn new(seed: u64, a: Brain, b: Brain) -> Match {
        let arena = Arena::generate(seed);
        let mut ra = Robot::new(Role::Hero);
        ra.x = SPAWN_A;
        ra.facing = 1.0;
        let mut rb = Robot::new(Role::Monster);
        rb.x = SPAWN_B;
        rb.facing = -1.0;
        ra.fk();
        rb.fk();
        ra.set_sword(ra.sword_capsule(), ra.sword_capsule());
        rb.set_sword(rb.sword_capsule(), rb.sword_capsule());
        let robots = [ra, rb];
        let prev_state = [robots[0].state(), robots[1].state()];
        Match {
            robots,
            arena,
            tick: 0,
            threats: Vec::new(),
            idle_guard_ticks: [0; 2],
            log: None,
            ctl: [make_ctl(a, seed, 1), make_ctl(b, seed, 2)],
            hitstop: 0,
            cooldown: [0; 2],
            turn: [0; 2],
            last_live: [None; 2],
            prev_state,
            prev_destroyed: [[false; 13]; 2],
            next_threat: 0,
            ended: false,
        }
    }
