//! Duel loop: two robots, brains, combat per tick, threat tracking, no forcing rule, outcome.
use crate::grace::GraceRec;
use crate::arena::Arena;
use crate::combat::{self, resolve_tick_with, ClashKind, CombatEvent, Mods};
use crate::executor::Executor;
use crate::moveset::{phase_durations, spec, MoveId, MoveParams, Phase};
use crate::rules::{self, RuleState};
use crate::skillbot::{is_attack, Request, SkillBot};
use crate::stylebot::{StyleBot, StyleId};
use crate::fitness::{self, ThreatOutcome, zone_weight};
use crate::config::*;
use crate::genome::Genome;
use crate::log::{self, f2, FightLog};
use crate::moves::raw_impact;
use crate::net::{decode, Net};
use crate::robot::{Capsule, Joint, Part, Robot, Role, State, STAND_TARGETS};
use crate::scripted::{self, Action, Scripted};
use crate::sense::{observe_moves, MoveInfo};

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
    /// Scripted skill tier 1 (Brawler), 2 (Reactive) or 3 (Duellist), driven through the Executor.
    Skill(u8),
    /// Scripted style teacher (press, counter, duelist), driven through the Executor.
    Style(StyleId),
    /// A net that acts while a shadow style teacher watches the same fight; the teacher's choices are only recorded
    /// (imitation with the student's own state distribution).
    Student(Genome, StyleId),
    /// Test support: walks in and requests this move whenever the executor can start one; never defends.
    Spam(MoveId),
    /// Test support: as `Spam`, requesting the listed moves in round-robin order.
    Cycle(Vec<MoveId>),
}

enum Ctl {
    /// Net, last decoded (request, drive).
    Net(Box<Net>, Request, f32),
    Skill(Box<SkillBot>),
    Style(Box<StyleBot>),
    Student(Box<Net>, Request, f32, Box<StyleBot>),
    /// Move list, next index, a request is pending acceptance.
    Cycle(Vec<MoveId>, usize, bool),
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
    pub end_tick: u32,
    pub followup_hit: bool,
    /// The swing's path met the defender's parts as they stood when the threat began.
    pub would_hit: bool,
    snapshot: Vec<Capsule>,
    start_x: f32,
    start_angles: [f32; 12],
    moved: f32,
    flexed: f32,
    quiet: u32,
    /// The attacker's move when the threat began, and whether the defender backstepped while it was live.
    att_move: MoveId,
    def_backstep: bool,
    /// Moves the attacker had started when the threat began (identifies the swing).
    swing: usize,
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
    pub ko: [bool; 2],
    /// Zone-weighted damage dealt by each fighter, over the opponent's total hp (0..1).
    pub dmg_dealt_norm: [f32; 2],
    /// Share of the match spent far apart with no hit exchanged recently.
    pub passivity: f32,
    /// Mean strike quality of each fighter's landed hits.
    pub chain_quality: [f32; 2],
    /// Body hits landed by each fighter.
    pub hits: [u32; 2],
    /// Mean per-tick |change of joint speed| / max speed, over joints and ticks, per fighter.
    pub jerk: [f32; 2],
    /// The emergency hard tick cap ended the fight (a mutual loss, scored far below any death).
    pub capped: bool,
    /// Training-only: summed stall units of all no-hit stretches (both sides), see `fitness::stall_units`.
    pub stall: f32,
    /// Attack moves started by each fighter.
    pub swings: [u32; 2],
    /// Hits each fighter landed on an open (staggered, fallen or rule-open) opponent.
    pub punish_hits: [u32; 2],
    /// Windows in which each fighter was open.
    pub open_windows: [u32; 2],
    /// Distinct attack moves each fighter landed a hit with.
    pub landed_moves: [u32; 2],
    /// Mean torso distance over the fight, metres.
    pub gap_mean: f32,
    /// Mean torso distance over the ticks the fighters were within 4 m of each other.
    pub engaged_gap: f32,
    /// Mean and longest combo (hits by one fighter with no hit by the other and gaps of at most `COMBO_GAP` ticks).
    pub combo_mean: [f32; 2],
    pub combo_max: [u32; 2],
    /// Attack moves started per fighter, by `MoveId` index.
    pub move_counts: [[u16; 14]; 2],
    /// Body hits landed per fighter, by the `MoveId` of the blow.
    pub hit_counts: [[u16; 14]; 2],
    /// Share of ticks each fighter's root barely moved (under 0.004 m per tick).
    pub still_share: [f32; 2],
    /// Share of ticks each fighter walked toward the opponent (at least 0.004 m per tick).
    pub fwd_share: [f32; 2],
    /// Std dev of the torso gap over the ticks within 4 m, metres.
    pub engaged_std: f32,
    /// Hits landed within `FOLLOW_TICKS` of this fighter parrying or dodging a threat (never counts the parry itself).
    pub counter_hits: [u32; 2],
    /// Hits landed within `FOLLOW_TICKS` of this fighter starting a Backstep.
    pub backstep_hits: [u32; 2],
}

/// Ticks after a parry, dodge or backstep within which a landed hit counts as converting it.
pub const FOLLOW_TICKS: u32 = 120;

/// Ticks after an attack move ends during which its blade can still land a blow.
pub const BLOW_LAG: u32 = 60;

/// Ticks between two hits of one fighter that still count as one combo.
pub const COMBO_GAP: u32 = 90;

/// Mean and longest combo of fighter `who` from the ordered hit list (tick, attacker).
pub fn combos(hit_list: &[(u32, u8)], who: u8) -> (f32, u32) {
    let (mut n, mut cur, mut best, mut total, mut last) = (0u32, 0u32, 0u32, 0u32, 0u32);
    for &(t, a) in hit_list {
        if a == who {
            if cur > 0 && t.saturating_sub(last) <= COMBO_GAP {
                cur += 1;
            } else {
                if cur > 0 { n += 1; total += cur; }
                cur = 1;
            }
            last = t;
            best = best.max(cur);
        } else if cur > 0 {
            n += 1;
            total += cur;
            cur = 0;
        }
    }
    if cur > 0 { n += 1; total += cur; }
    (if n == 0 { 0.0 } else { total as f32 / n as f32 }, best)
}

/// First-order low-pass of an action vector: `prev + alpha * (target - prev)`.
pub fn smooth_toward(prev: &[f32; 12], target: &[f32; 12], alpha: f32) -> [f32; 12] {
    let mut o = [0.0f32; 12];
    for i in 0..12 {
        o[i] = prev[i] + alpha * (target[i] - prev[i]);
    }
    o
}

const THREAT_SPEED: f32 = 2.5;
const THREAT_RANGE: f32 = 0.25;
const THREAT_HORIZON: f32 = 0.4;
const SWING_COOLDOWN: u32 = 12;
const IDLE_GUARD_WINDOW: u32 = 36;
const TURN_TICKS: u32 = 15;

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

/// Gap between two capsules (negative when they overlap).
fn cap_dist(a: &Capsule, b: &Capsule) -> f32 {
    seg_seg_dist(a.a, a.b, b) - a.r
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
    /// Last raw net output per fighter (targets, before smoothing); diagnostics only.
    pub raw_action: [[f32; 12]; 2],
    /// Targets actually applied to the motors on the last tick.
    pub applied_action: [[f32; 12]; 2],
    dealt_w: [f32; 2],
    quality_sum: [f32; 2],
    hits: [u32; 2],
    jerk_sum: [f32; 2],
    adv_ticks: u32,
    capped: bool,
    stretch_start: u32,
    stall_acc: f32,
    last_hit: Option<u32>,
    passive_ticks: u32,
    pub log: Option<FightLog>,
    ctl: [Ctl; 2],
    hitstop: u32,
    cooldown: [u32; 2],
    /// Last tick each fighter was running an attack move (the blade keeps moving a few ticks after the move ends).
    last_attack_tick: [u32; 2],
    turn: [u32; 2],
    last_live: [Option<u32>; 2],
    prev_state: [State; 2],
    prev_destroyed: [[bool; 13]; 2],
    next_threat: u32,
    ended: bool,
    exec: [Executor; 2],
    beat_blade: [bool; 2],
    /// Blade-blade clash events resolved so far (measurement).
    pub clash_count: u32,
    rules: [RuleState; 2],
    starts: [Vec<MoveId>; 2],
    last_cost: [f32; 2],
    prev_phase: [Phase; 2],
    prev_id: [MoveId; 2],
    prev_cancel: [bool; 2],
    contact_seen: [bool; 2],
    bash_done: [bool; 2],
    whiff_pending: [bool; 2],
    /// Measured-only grace records (never read by fitness).
    pub grace: [GraceRec; 2],
    prev_open: [bool; 2],
    grace_on: bool,
    last_feint: [Option<u32>; 2],
    hit_mask: [u32; 2],
    open_windows: [u32; 2],
    hit_list: Vec<(u32, u8)>,
    gap_total: f64,
    engaged_total: f64,
    engaged_ticks: u32,
    engaged_sq: f64,
    hit_moves: [[u16; 14]; 2],
    still_ticks: [u32; 2],
    fwd_ticks: [u32; 2],
    backstep_ticks: [Vec<u32>; 2],
    /// Imitation recording of one fighter's controller (see `Recorder`).
    pub recorder: Option<Recorder>,
}

/// One imitation sample (drive is stored in the fighter's own frame: positive = toward the way it faces): what the fighter sensed on a decision tick and what the teacher asked for in that window.
#[derive(Clone, Debug)]
pub struct Sample {
    pub x: Vec<f32>,
    pub mv: u8,
    pub params: [f32; 4],
    pub drive: f32,
    pub n: u8,
}

/// Records the teacher's choices for fighter `side`, one sample per decision tick.
#[derive(Clone, Debug)]
pub struct Recorder {
    pub side: usize,
    pub samples: Vec<Sample>,
}

fn make_ctl(b: Brain, seed: u64, salt: u64) -> Ctl {
    match b {
        Brain::Net(g) => Ctl::Net(Box::new(Net::from_genome(&g)), None, 0.0),
        Brain::Skill(t) => Ctl::Skill(Box::new(SkillBot::new(seed.wrapping_add(salt), t))),
        Brain::Student(g, st) => Ctl::Student(Box::new(Net::from_genome(&g)), None, 0.0, Box::new(StyleBot::new(st, seed.wrapping_add(salt)))),
        Brain::Style(st) => Ctl::Style(Box::new(StyleBot::new(st, seed.wrapping_add(salt)))),
        Brain::Spam(m) => Ctl::Cycle(vec![m], 0, false),
        Brain::Cycle(l) => Ctl::Cycle(if l.is_empty() { vec![MoveId::Idle] } else { l }, 0, false),
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
            raw_action: [STAND_TARGETS; 2],
            applied_action: [STAND_TARGETS; 2],
            dealt_w: [0.0; 2],
            quality_sum: [0.0; 2],
            hits: [0; 2],
            jerk_sum: [0.0; 2],
            adv_ticks: 0,
            capped: false,
            stretch_start: 0,
            stall_acc: 0.0,
            last_hit: None,
            passive_ticks: 0,
            log: None,
            ctl: [make_ctl(a, seed, 1), make_ctl(b, seed, 2)],
            hitstop: 0,
            cooldown: [0; 2],
            last_attack_tick: [0; 2],
            turn: [0; 2],
            last_live: [None; 2],
            prev_state,
            prev_destroyed: [[false; 13]; 2],
            next_threat: 0,
            ended: false,
            exec: [Executor::new(), Executor::new()],
            beat_blade: [false; 2],
            clash_count: 0,
            rules: [RuleState::new(), RuleState::new()],
            starts: [Vec::new(), Vec::new()],
            last_cost: [0.0; 2],
            prev_phase: [Phase::Idle; 2],
            prev_id: [MoveId::Idle; 2],
            prev_cancel: [false; 2],
            contact_seen: [false; 2],
            bash_done: [false; 2],
            whiff_pending: [false; 2],
            grace: [GraceRec::default(), GraceRec::default()],
            prev_open: [false; 2],
            grace_on: true,
            last_feint: [None; 2],
            hit_mask: [0; 2],
            open_windows: [0; 2],
            hit_list: Vec::new(),
            gap_total: 0.0,
            engaged_total: 0.0,
            engaged_ticks: 0,
            engaged_sq: 0.0,
            hit_moves: [[0; 14]; 2],
            still_ticks: [0; 2],
            fwd_ticks: [0; 2],
            backstep_ticks: [Vec::new(), Vec::new()],
            recorder: None,
        }
    }

    /// Hits so far as (tick, attacker).
    pub fn hit_log(&self) -> &[(u32, u8)] {
        &self.hit_list
    }

    pub fn enable_log(&mut self) {
        if self.log.is_none() {
            self.log = Some(FightLog::default());
        }
    }

    pub fn is_over(&self) -> bool {
        self.ended || self.tick >= HARD_CAP_TICKS || self.robots.iter().any(|r| r.is_ko())
    }

    /// Fighter `i` is in a blow: running an attack move, or within `BLOW_LAG` ticks of one (raw controllers always count).
    pub fn striking(&self, i: usize) -> bool {
        let raw = matches!(self.ctl[i], Ctl::Dummy | Ctl::Fixed(_) | Ctl::Scripted(_));
        raw || self.hitstop > 0 || is_attack(self.exec[i].move_id()) || self.tick <= self.last_attack_tick[i] + BLOW_LAG
    }

    /// `striking` as it will read once this tick has been counted (the fix-up runs before the tick counter advances).
    fn striking_next(&self, i: usize) -> bool {
        let raw = matches!(self.ctl[i], Ctl::Dummy | Ctl::Fixed(_) | Ctl::Scripted(_));
        raw || self.hitstop > 0 || is_attack(self.exec[i].move_id()) || self.tick + 1 <= self.last_attack_tick[i] + BLOW_LAG
    }

    /// Fighter `i`'s sword overlaps a body part of the opponent.
    pub fn sword_in_body(&self, i: usize) -> bool {
        let (me, foe) = (&self.robots[i], &self.robots[1 - i]);
        if !me.has_sword() || matches!(me.state(), State::Fallen | State::Ko) || matches!(foe.state(), State::Fallen | State::Ko) {
            return false;
        }
        let s = me.sword_capsule();
        Part::ALL.iter().any(|p| !matches!(p, Part::UArmR | Part::FArmR) && !foe.is_detached(*p) && cap_dist(&s, &foe.capsule(*p)) < 0.0)
    }

    fn emit(&mut self, e: String) {
        if let Some(l) = self.log.as_mut() {
            l.events.push(e);
        }
    }

    /// Cumulative executor energy of `who` (measurement only).
    pub fn exec_energy(&self, who: usize) -> f32 {
        self.exec[who].energy_spent()
    }

    /// Zone-weighted damage dealt by `who` (measurement only).
    pub fn damage_dealt_weighted(&self, who: usize) -> f32 {
        self.dealt_w[who]
    }

    /// Skip the per-tick grace trace (training does not read it; the counters stay cheap and on).
    pub fn disable_grace_trace(&mut self) {
        self.grace_on = false;
    }

    fn track_grace(&mut self) {
        for i in 0..2 {
            let open = self.rules[i].is_open() || matches!(self.robots[i].state(), State::Staggered | State::Fallen);
            if open && !self.prev_open[i] {
                self.grace[i].open_windows += 1;
                self.open_windows[i] += 1;
            }
            self.prev_open[i] = open;
        }
        if !self.grace_on {
            return;
        }
        let gap = (self.robots[0].x - self.robots[1].x).abs() as f64;
        for i in 0..2 {
            let tip = self.robots[i].sword_capsule().b;
            self.grace[i].tip.push(tip);
            self.grace[i].gap_n += 1;
            self.grace[i].gap_sum += gap;
            self.grace[i].gap_sq += gap * gap;
        }
    }

    /// Every move the executor of `who` accepted, in order (Feint included).
    pub fn move_starts(&self, who: usize) -> Vec<MoveId> {
        self.starts[who].clone()
    }

    /// What the executor and rules know about fighter `i`'s current move.
    pub fn move_info(&self, i: usize) -> MoveInfo {
        let e = &self.exec[i];
        let r = &self.rules[i];
        let rec = if e.phase() == Phase::Recovery { (1.0 - e.progress()) * phase_durations(e.move_id(), &e.params())[2] } else { 0.0 };
        MoveInfo {
            id: e.move_id(),
            phase: e.phase(),
            progress: e.progress(),
            committed: e.committed(),
            open: r.is_open(),
            riposte_remaining: r.riposte_remaining(),
            guard_broken_remaining: r.guard_broken_remaining(),
            last_cost: self.last_cost[i],
            recovery_remaining: rec,
            last_move: self.starts[i].last().copied().unwrap_or(MoveId::Idle),
        }
    }

    fn decide(&mut self, i: usize) -> Action {
        let mine = self.move_info(i);
        let theirs = self.move_info(1 - i);
        let live = self.last_live[i].map_or(false, |t| self.tick.saturating_sub(t) <= 6);
        let tick = self.tick;
        let (me, opp) = (&self.robots[i], &self.robots[1 - i]);
        let mut shadow: Option<(Request, f32)> = None;
        let (req, drive): (Request, f32) = match &mut self.ctl[i] {
            Ctl::Dummy => return scripted::dummy(),
            Ctl::Fixed(a) => return *a,
            Ctl::Scripted(s) => return s.act(me, opp, PHYS_DT),
            Ctl::Net(net, held, hdrive) => {
                let mut req = None;
                if tick % DECISION_TICKS == 0 {
                    let x = observe_moves(me, opp, &self.arena, &mine, &theirs);
                    let d = decode(&net.forward(&x));
                    *held = if d.mv == MoveId::Idle { None } else { Some((d.mv, d.params)) };
                    *hdrive = d.drive;
                    req = *held;
                }
                (req, *hdrive)
            }
            Ctl::Skill(b) => b.act(me, opp, &mine, &theirs, live, PHYS_DT),
            Ctl::Style(b) => b.act(me, opp, &mine, &theirs, live, PHYS_DT),
            Ctl::Student(net, held, hdrive, teacher) => {
                shadow = Some(teacher.act(me, opp, &mine, &theirs, live, PHYS_DT));
                let mut req = None;
                if tick % DECISION_TICKS == 0 {
                    let x = observe_moves(me, opp, &self.arena, &mine, &theirs);
                    let d = decode(&net.forward(&x));
                    *held = if d.mv == MoveId::Idle { None } else { Some((d.mv, d.params)) };
                    *hdrive = d.drive;
                    req = *held;
                }
                (req, *hdrive)
            }
            Ctl::Cycle(list, k, pending) => {
                if *pending && mine.phase != Phase::Idle {
                    *k += 1;
                    *pending = false;
                }
                let dx = opp.x - me.x;
                let drive = if dx.abs() > 2.2 { dx.signum() } else { 0.0 };
                let mut req = None;
                if mine.phase == Phase::Idle && dx.abs() < 3.0 {
                    req = Some((list[*k % list.len()], MoveParams { reach: 0.5, speed: 0.5, arc: 0.5, delay: 0.0 }));
                    *pending = true;
                }
                (req, drive)
            }
        };
        if self.recorder.as_ref().map_or(false, |r| r.side == i) {
            let x = if tick % DECISION_TICKS == 0 { Some(observe_moves(&self.robots[i], &self.robots[1 - i], &self.arena, &mine, &theirs)) } else { None };
            let (req, drive) = shadow.unwrap_or((req, drive));
            let facing = if self.robots[i].facing >= 0.0 { 1.0 } else { -1.0 };
            let rec = self.recorder.as_mut().unwrap();
            if let Some(x) = x {
                rec.samples.push(Sample { x: x.to_vec(), mv: 0, params: [0.5; 4], drive: 0.0, n: 0 });
            }
            if let Some(s) = rec.samples.last_mut() {
                if s.n < DECISION_TICKS as u8 {
                    if s.mv == 0 {
                        if let Some((m, pr)) = req {
                            s.mv = m.index() as u8;
                            s.params = [pr.reach, pr.speed, pr.arc, pr.delay];
                        }
                    }
                    s.drive += drive * facing;
                    s.n += 1;
                }
            }
        }
        let riposte = self.rules[i].riposte_open();
        let before = self.exec[i].energy_spent();
        self.exec[i].set_threat(live);
        self.exec[i].set_riposte_open(riposte);
        let opp_down = self.robots[1 - i].state() == State::Fallen;
        self.exec[i].set_target_down(opp_down);
        let act = self.exec[i].step(&self.robots[i], req, PHYS_DT);
        let delta = self.exec[i].energy_spent() - before;
        if delta > 0.0 {
            let r = &mut self.robots[i];
            r.energy = (r.energy - delta).clamp(0.0, ENERGY_MAX);
            if delta > 0.5 {
                self.last_cost[i] = delta;
            }
        }
        self.raw_action[i] = act.targets;
        self.after_step(i);
        Action { targets: act.targets, drive: (drive + act.drive).clamp(-1.0, 1.0) }
    }

    /// Track move starts, feints and the Active -> Recovery edge for fighter `i`.
    fn after_step(&mut self, i: usize) {
        let (ph, id, cn) = (self.exec[i].phase(), self.exec[i].move_id(), self.exec[i].cancelling());
        let t = self.tick;
        // Same window the clash resolution counts as a parry in progress.
        self.robots[i].parrying = id == MoveId::Parry
            && (matches!(ph, Phase::Windup | Phase::Active) || (ph == Phase::Recovery && self.exec[i].progress() < 0.5));
        if cn && !self.prev_cancel[i] {
            self.starts[i].push(MoveId::Feint);
            self.grace[i].feints += 1;
            self.last_feint[i] = Some(t);
            self.emit(format!("{{\"t\":{},\"kind\":\"feint\",\"who\":{}}}", t, i));
        } else if ph == Phase::Windup && !cn && (self.prev_phase[i] != Phase::Windup || self.prev_id[i] != id) {
            self.starts[i].push(id);
            if id == MoveId::Backstep {
                self.backstep_ticks[i].push(t);
            }
            self.contact_seen[i] = false;
            self.beat_blade[i] = false;
            self.bash_done[i] = false;
            let pr = self.exec[i].params();
            self.emit(format!(
                "{{\"t\":{},\"kind\":\"move\",\"who\":{},\"name\":\"{}\",\"reach\":{},\"speed\":{},\"arc\":{},\"delay\":{}}}",
                t, i, id.name(), f2(pr.reach), f2(pr.speed), f2(pr.arc), f2(pr.delay)
            ));
        }
        if self.prev_phase[i] == Phase::Active && ph == Phase::Recovery && !cn {
            self.whiff_pending[i] = true;
        }
        self.prev_phase[i] = ph;
        self.prev_id[i] = id;
        self.prev_cancel[i] = cn;
    }

    /// After combat: a move whose Active ended with no contact is a whiff and leaves its user Open.
    fn resolve_whiffs(&mut self) {
        for i in 0..2 {
            if !self.whiff_pending[i] {
                continue;
            }
            self.whiff_pending[i] = false;
            let id = self.exec[i].move_id();
            if spec(id).can_whiff && !self.contact_seen[i] {
                let rec = phase_durations(id, &self.exec[i].params())[2];
                let t = self.tick;
                self.grace[i].whiffs += 1;
                if let Some(ft) = self.last_feint[1 - i] {
                    if t.saturating_sub(ft) <= 40 {
                        self.grace[1 - i].baits += 1;
                        self.last_feint[1 - i] = None;
                    }
                }
                self.emit(format!("{{\"t\":{},\"kind\":\"whiff\",\"who\":{},\"name\":\"{}\"}}", t, i, id.name()));
                if id == MoveId::Parry {
                    self.rules[i].on_parry_miss();
                } else {
                    self.rules[i].begin_recovery(true, rec);
                }
            }
        }
    }

    /// Kick and shield bash: contacts the sword sweep does not model.
    fn special_contacts(&mut self) {
        for att in 0..2 {
            let def = 1 - att;
            let e = &self.exec[att];
            if e.phase() != Phase::Active || self.bash_done[att] || self.robots[def].is_ko() {
                continue;
            }
            let id = e.move_id();
            let t = self.tick;
            match id {
                MoveId::ShieldBash => {
                    let sh = self.robots[att].shield_capsule();
                    let d = &self.robots[def];
                    let mut hit = Part::ALL.iter().any(|p| {
                        matches!(p, Part::Head | Part::Torso | Part::Pelvis | Part::UArmL | Part::UArmR) && !d.is_detached(*p) && cap_dist(&sh, &d.capsule(*p)) <= 0.05
                    });
                    if d.shield_active() && cap_dist(&sh, &d.shield_capsule()) <= 0.05 {
                        hit = true;
                    }
                    if !hit {
                        continue;
                    }
                    self.bash_done[att] = true;
                    self.contact_seen[att] = true;
                    self.exec[att].notify_contact();
                    let guarding = self.robots[def].guard_raised() || (self.exec[def].move_id() == MoveId::Guard && self.exec[def].phase() != Phase::Idle);
                    if guarding {
                        let (r, st) = (&mut self.robots[def], &mut self.rules[def]);
                        rules::guard_break(r, st);
                        self.emit(format!("{{\"t\":{},\"kind\":\"guard_break\",\"who\":{},\"victim\":{}}}", t, att, def));
                    } else {
                        self.robots[def].add_posture(15.0);
                    }
                }
                MoveId::Kick => {
                    let foot = self.robots[att].capsule(Part::FootR);
                    let d = &self.robots[def];
                    let hit = [Part::Torso, Part::Pelvis, Part::ThighL, Part::ThighR].iter().any(|p| !d.is_detached(*p) && cap_dist(&foot, &d.capsule(*p)) <= 0.05);
                    if hit {
                        self.bash_done[att] = true;
                        self.contact_seen[att] = true;
                        self.exec[att].notify_contact();
                        self.robots[def].add_posture(20.0);
                    }
                }
                _ => {}
            }
        }
    }

    /// Arena walls (lo, hi). Fixed for the whole fight.
    pub fn bounds(&self) -> (f32, f32) {
        (0.5, self.arena.width - 0.5)
    }

    /// End the match as if the emergency cap fired (tests).
    pub fn force_cap_for_test(&mut self) -> MatchResult {
        self.capped = true;
        self.ended = true;
        self.finish_threats();
        self.result()
    }

    fn blocked_by_obstacle(&self, x: f32) -> bool {
        self.arena.obstacles.iter().any(|o| (x - o.x).abs() < o.w * 0.5 + 0.35)
    }

    /// Advance one physics tick.
    pub fn step(&mut self) {
        if self.is_over() {
            self.ended = true;
            return;
        }
        if self.hitstop > 0 {
            self.hitstop -= 1;
            if self.hitstop == 0 {
                let chains = [self.robots[0].sword_chain(), self.robots[1].sword_chain()];
                self.keep_swords_out_of_bodies(&chains);
            }
        } else {
            self.adv_ticks += 1;
            let gap = (self.robots[0].x - self.robots[1].x).abs();
            self.gap_total += gap as f64;
            if gap < 4.0 {
                self.engaged_total += gap as f64;
                self.engaged_sq += (gap as f64) * (gap as f64);
                self.engaged_ticks += 1;
            }
            let side = if self.robots[1].x >= self.robots[0].x { 1.0f32 } else { -1.0 };
            let chains = [self.robots[0].sword_chain(), self.robots[1].sword_chain()];
            self.advance_fighters();
            self.update_threats_pre();
            self.run_combat();
            self.break_mutual_stagger();
            self.separate_bodies(side);
            self.keep_blades_apart(&chains);
            self.special_contacts();
            self.resolve_whiffs();
            self.keep_swords_out_of_bodies(&chains);
            for r in self.rules.iter_mut() {
                r.tick(PHYS_DT);
            }
            self.track_grace();
        }
        self.update_threats_post();
        self.track_idle_guard();
        self.track_passivity();
        self.track_states();
        if self.tick % 2 == 0 {
            if self.log.is_some() {
                let f = log::frame_json(self);
                if let Some(l) = self.log.as_mut() {
                    l.frames.push(f);
                }
            }
        }
        self.tick += 1;
        if self.tick >= HARD_CAP_TICKS && !self.robots.iter().any(|r| r.is_ko()) {
            self.capped = true;
        }
        if self.tick >= HARD_CAP_TICKS || self.robots.iter().any(|r| r.is_ko()) {
            // The log always ends on the final state.
            if self.tick % 2 == 0 && self.log.is_some() {
                let f = log::frame_json(self);
                if let Some(l) = self.log.as_mut() {
                    l.frames.push(f);
                }
            }
            self.ended = true;
            self.finish_threats();
        }
    }

    fn advance_fighters(&mut self) {
        for i in 0..2 {
            // Turn to face the opponent (costs 0.25 s, not while swinging or down).
            let want = if self.robots[1 - i].x >= self.robots[i].x { 1.0 } else { -1.0 };
            let r = &mut self.robots[i];
            if r.facing != want && r.can_act() && r.sword_moving_since == 0.0 {
                self.turn[i] += 1;
                if self.turn[i] >= TURN_TICKS {
                    r.facing = want;
                    r.fk();
                    self.turn[i] = 0;
                }
            } else {
                self.turn[i] = 0;
            }
        }
        let (lo, hi) = self.bounds();
        for i in 0..2 {
            let act = self.decide(i);
            self.applied_action[i] = act.targets;
            let old_x = self.robots[i].x;
            if self.exec[i].phase() != Phase::Idle {
                self.robots[i].clear_recoil();
            }
            let r = &mut self.robots[i];
            let before = r.joint_vel;
            r.apply_targets(&act.targets, PHYS_DT);
            let mut jerk = 0.0;
            for j in Joint::ALL {
                let k = j as usize;
                jerk += (r.joint_vel[k] - before[k]).abs() / j.max_speed();
            }
            self.jerk_sum[i] += jerk / 12.0;
            r.apply_drive(act.drive, PHYS_DT, self.arena.width);
            let r = &mut self.robots[i];
            if r.x < lo || r.x > hi {
                r.x = r.x.clamp(lo, hi);
                r.fk();
            }
            let nx = self.robots[i].x;
            if nx != old_x && self.blocked_by_obstacle(nx) && !self.blocked_by_obstacle(old_x) {
                self.robots[i].x = old_x;
                self.robots[i].fk();
            }
        }
    }

    /// Fighter `k`'s blade, held where it is now, meets the other blade's path this tick.
    fn blade_conflict_with_foe(&self, k: usize) -> bool {
        let (me, foe) = (&self.robots[k], &self.robots[1 - k]);
        let c = me.sword_capsule();
        combat::blades_conflict(c, c, foe.sword_prev_capsule(), foe.sword_capsule())
    }

    fn blades_cross_at_end(&self) -> bool {
        let (a, b) = (self.robots[0].sword_capsule(), self.robots[1].sword_capsule());
        combat::blades_conflict(a, a, b, b)
    }

    fn blades_cross(&self) -> bool {
        let (a, b) = (&self.robots[0], &self.robots[1]);
        combat::blades_conflict(a.sword_prev_capsule(), a.sword_capsule(), b.sword_prev_capsule(), b.sword_capsule())
    }

    /// Blades are solid: a tick never ends with one blade through the other (nor having crossed it on the way).
    /// A held, undriven blade that a swing runs into is batted aside (the swing carries on); any other meeting stops
    /// both blades where they were at the start of the tick (then, if the bodies' walking still causes it, the
    /// walking too). The same rule for both fighters. A blade lying with a fallen or dead fighter is not in play.
    fn keep_blades_apart(&mut self, chains: &[crate::robot::SwordChain; 2]) {
        if !self.robots.iter().all(|r| r.has_sword() && !matches!(r.state(), State::Fallen | State::Ko)) || !self.blades_cross() {
            return;
        }
        let (p0, p1) = (self.robots[0].sword_prev_capsule(), self.robots[1].sword_prev_capsule());
        let started_clear = !combat::blades_conflict(p0, p0, p1, p1);
        // Blade speed including the walk: a fighter who walks into a blade is moving it.
        let speed = [0, 1].map(|i| {
            let (p, c) = (self.robots[i].sword_prev_capsule(), self.robots[i].sword_capsule());
            let d = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() / PHYS_DT;
            // A blade coming back from being batted aside is not driving anything: it yields again.
            if self.robots[i].in_recoil() { 0.0 } else { d(p.a, c.a).max(d(p.b, c.b)) }
        });
        // The slower blade yields: whatever is moving into a blade moves it aside (a held blade, a slower swing), and a
        // blade held in a parry does not yield. Exactly equal speeds: alternate by tick.
        let batable = [0, 1].map(|i| !self.robots[i].parrying);
        let first = if speed[0] == speed[1] { (self.tick % 2) as usize } else if speed[0] < speed[1] { 0 } else { 1 };
        let order = [first, 1 - first];
        let apart = |m: &Self| if started_clear { !m.blades_cross() } else { !m.blades_cross_at_end() };
        // 1. Bat the slower blade aside (swept clear).
        for k in order {
            if batable[k] && self.bat_clear(k, &apart) {
                return;
            }
        }
        // 2. Otherwise both blades stop where they were at the start of the tick (and, if the walking or a push is
        // what brought them together, the bodies too).
        if started_clear {
            for i in 0..2 {
                self.robots[i].restore_sword_chain(&chains[i], false);
            }
            if apart(self) {
                return;
            }
            for i in 0..2 {
                self.robots[i].restore_sword_chain(&chains[i], true);
            }
            if apart(self) {
                return;
            }
        }
        // 3. Last resort (the tick began overlapped, e.g. a fighter got up over a blade): bat either blade clear.
        for k in order {
            if self.bat_clear(k, &apart) {
                return;
            }
        }
    }

    /// Bodies are solid to a sword that is not in a blow: a resting, guarding or parrying blade is pushed out of the
    /// opponent's body (batted aside, else rolled back to where it was). Same rule for both fighters.
    fn keep_swords_out_of_bodies(&mut self, chains: &[crate::robot::SwordChain; 2]) {
        for k in 0..2 {
            if self.striking_next(k) || !self.sword_in_body(k) {
                continue;
            }
            // First choice: the blade stops against the body where the tick began.
            self.robots[k].restore_sword_chain(&chains[k], false);
            if !self.sword_in_body(k) && !self.blade_conflict_with_foe(k) {
                let cur = self.robots[k].sword_capsule();
                self.robots[k].set_sword(cur, cur);
                continue;
            }
            let apart = move |m: &Self| !m.sword_in_body(k) && !m.blade_conflict_with_foe(k);
            if self.bat_clear(k, &apart) {
                continue;
            }
            let before = self.robots[k].sword_chain();
            let mut cleared = false;
            'e: for step in 1..=14 {
                for sign in [1.0f32, -1.0] {
                    for sh in [0.0f32, 0.3, -0.3] {
                        self.robots[k].restore_sword_chain(&before, false);
                        self.robots[k].bat_elbow(sign * 0.1 * step as f32);
                        if sh != 0.0 {
                            self.robots[k].bat_sword_aside(sh);
                        }
                        if !self.sword_in_body(k) && !self.blade_conflict_with_foe(k) {
                            cleared = true;
                            break 'e;
                        }
                    }
                }
            }
            if !cleared {
                self.robots[k].restore_sword_chain(&chains[k], false);
            }
            let cur = self.robots[k].sword_capsule();
            self.robots[k].set_sword(cur, cur);
        }
    }

    /// Rotate fighter `k`'s right shoulder in growing steps, either way, until `apart` holds; false (and the blade left
    /// as it was) if no angle in reach works.
    fn bat_clear(&mut self, k: usize, apart: &dyn Fn(&Self) -> bool) -> bool {
        let before = self.robots[k].sword_chain();
        let prev = self.robots[k].sword_prev_capsule();
        for step in 1..=24 {
            for sign in [1.0f32, -1.0] {
                self.robots[k].restore_sword_chain(&before, false);
                self.robots[k].bat_sword_aside(sign * 0.1 * step as f32);
                // A batted blade is put aside, not swept through the other: it counts from where it lands.
                let landed = self.robots[k].sword_capsule();
                self.robots[k].set_sword(landed, landed);
                if apart(self) {
                    return true;
                }
            }
        }
        self.robots[k].restore_sword_chain(&before, false);
        self.robots[k].clear_batted();
        let cur = self.robots[k].sword_capsule();
        self.robots[k].set_sword(prev, cur);
        false
    }

    /// Bodies are solid: if the torsos are closer than `BODY_MIN_SEP` (or have crossed), push both apart by the same
    /// amount, keeping the side each started the tick on. A fighter pinned by a wall or obstacle yields the rest to the other.
    fn separate_bodies(&mut self, side: f32) {
        let d = (self.robots[1].x - self.robots[0].x) * side;
        if d >= BODY_MIN_SEP {
            return;
        }
        let push = (BODY_MIN_SEP - d) * 0.5;
        let (lo, hi) = self.bounds();
        let mut moved = [0.0f32; 2];
        for (i, dir) in [(0usize, -side), (1usize, side)] {
            let old = self.robots[i].x;
            let nx = (old + dir * push).clamp(lo, hi);
            if !self.blocked_by_obstacle(nx) || self.blocked_by_obstacle(old) {
                self.robots[i].x = nx;
            }
            moved[i] = (self.robots[i].x - old).abs();
        }
        // Whatever one side could not give, the other gives.
        let short = push * 2.0 - moved[0] - moved[1];
        if short > 1e-6 {
            let i = if moved[0] < moved[1] { 1 } else { 0 };
            let dir = if i == 0 { -side } else { side };
            let old = self.robots[i].x;
            let nx = (old + dir * short).clamp(lo, hi);
            if !self.blocked_by_obstacle(nx) || self.blocked_by_obstacle(old) {
                self.robots[i].x = nx;
            }
        }
        self.robots[0].fk();
        self.robots[1].fk();
    }

    fn tip_state(&self, att: usize) -> ([f32; 2], [f32; 2]) {
        let s = self.robots[att].sword_capsule();
        let p = self.robots[att].sword_prev_capsule();
        (s.b, [(s.b[0] - p.b[0]) / PHYS_DT, (s.b[1] - p.b[1]) / PHYS_DT])
    }

    fn update_threats_pre(&mut self) {
        for att in 0..2 {
            let def = 1 - att;
            if !self.robots[att].has_sword() || !self.robots[att].can_act() {
                continue;
            }
            let (tip, vel) = self.tip_state(att);
            let sp = self.robots[att].sword_prev_capsule();
            let sc = self.robots[att].sword_capsule();
            let live = is_live_threat(tip, vel, &self.robots[def]);
            if live {
                self.last_live[def] = Some(self.tick);
            }
            let speed = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
            let pot = raw_impact(35.0, speed, 1.0, 1.0, 1.0, 1.0);
            let active = self.threats.iter().position(|t| t.attacker == att && t.resolution.is_none());
            match active {
                None => {
                    if live && self.cooldown[att] == 0 {
                        self.create_threat(att, pot);
                    }
                }
                Some(k) => {
                    let (dx, angles) = {
                        let d = &self.robots[def];
                        let mut a = [0.0f32; 12];
                        for j in Joint::ALL {
                            a[j as usize] = d.joint_angle(j);
                        }
                        (d.x, a)
                    };
                    let t = &mut self.threats[k];
                    t.potential = t.potential.max(pot);
                    if self.exec[def].move_id() == MoveId::Backstep {
                        t.def_backstep = true;
                    }
                    if !t.would_hit && would_connect(sp, sc, &t.snapshot) {
                        t.would_hit = true;
                    }
                    t.moved = t.moved.max((dx - t.start_x).abs());
                    for j in 0..12 {
                        t.flexed = t.flexed.max((angles[j] - t.start_angles[j]).abs());
                    }
                    if live {
                        t.quiet = 0;
                    } else {
                        t.quiet += 1;
                    }
                }
            }
        }
    }

    fn create_threat(&mut self, att: usize, potential: f32) -> usize {
        let def = 1 - att;
        let d = &self.robots[def];
        let mut a = [0.0f32; 12];
        for j in Joint::ALL {
            a[j as usize] = d.joint_angle(j);
        }
        let t = Threat {
            id: self.next_threat,
            attacker: att,
            defender: def,
            start_tick: self.tick,
            resolution: None,
            potential,
            end_tick: 0,
            followup_hit: false,
            would_hit: false,
            snapshot: Part::ALL.iter().filter(|p| !d.is_detached(**p)).map(|p| d.capsule(*p)).collect(),
            start_x: d.x,
            start_angles: a,
            moved: 0.0,
            flexed: 0.0,
            quiet: 0,
            att_move: self.exec[att].move_id(),
            def_backstep: self.exec[def].move_id() == MoveId::Backstep,
            swing: self.starts[att].len(),
        };
        self.next_threat += 1;
        self.threats.push(t);
        self.threats.len() - 1
    }

    fn resolve_threat(&mut self, att: usize, res: Resolution) {
        let k = match self.threats.iter().position(|t| t.attacker == att && t.resolution.is_none()) {
            Some(k) => k,
            None => {
                // A second hit of a swing that already landed is the same threat, not a new one.
                let n = self.starts[att].len();
                if res == Resolution::Hit && self.threats.iter().rev().find(|t| t.attacker == att).map_or(false, |t| t.swing == n && t.resolution == Some(Resolution::Hit)) {
                    return;
                }
                let k = self.create_threat(att, 0.0);
                k
            }
        };
        self.threats[k].resolution = Some(res);
        self.threats[k].end_tick = self.tick;
    }

    fn run_combat(&mut self) {
        for att in 0..2 {
            if is_attack(self.exec[att].move_id()) {
                self.last_attack_tick[att] = self.tick + 1;
            }
            if self.cooldown[att] > 0 {
                self.cooldown[att] -= 1;
                continue;
            }
            let def = 1 - att;
            let att_id = self.exec[att].move_id();
            // Only attack moves hurt: a Parry, Guard or Backstep sweep through a body is no blow (same for both fighters).
            // The blade lags the executor, so a blow still counts for BLOW_LAG ticks after the attack move ends.
            let raw = matches!(self.ctl[att], Ctl::Dummy | Ctl::Fixed(_) | Ctl::Scripted(_));
            if !raw && !is_attack(att_id) && self.tick > self.last_attack_tick[att] + BLOW_LAG {
                continue;
            }
            let def_wa = matches!(self.exec[def].phase(), Phase::Windup | Phase::Active) && !self.exec[def].cancelling();
            let staggered = matches!(self.robots[def].state(), State::Staggered | State::Fallen);
            let finisher = self.exec[att].plunging() && self.robots[def].state() == State::Fallen;
            let total = rules::punish_mult_downed(&self.rules[def], def_wa, staggered, att_id == MoveId::Riposte, finisher);
            let mods = Mods {
                impact_mult: total / if staggered { 1.5 } else { 1.0 },
                posture_mult: if self.rules[def].is_open() { rules::OPEN_POSTURE_MULT } else { 1.0 },
                block_posture_mult: rules::heavy_block_posture(att_id, 1.0),
                shield_ignored: self.rules[def].guard_broken_remaining() > 0.0,
                ignore_blade: self.beat_blade[att],
            };
            let mods = if self.beat_blade[att] { Mods { impact_mult: mods.impact_mult * crate::config::BEAT_IMPACT, ..mods } } else { mods };
            let evs = {
                let (l, r) = self.robots.split_at_mut(1);
                let (a, b) = if att == 0 { (&mut l[0], &mut r[0]) } else { (&mut r[0], &mut l[0]) };
                resolve_tick_with(a, b, &self.arena, &mods)
            };
            if evs.is_empty() {
                continue;
            }
            self.clash_count += evs.iter().filter(|e| matches!(e, CombatEvent::Clash { .. })).count() as u32;
            // A swing that only met a held blade beats it aside and keeps going: no cooldown, no more clashes with it.
            if evs.len() == 1 && matches!(evs[0], CombatEvent::Clash { kind: ClashKind::Clash, .. }) && !self.robots[att].parrying && combat::is_inert(&self.robots[def]) {
                self.beat_blade[att] = true;
                let t = self.tick;
                if let CombatEvent::Clash { point, normal, .. } = &evs[0] {
                    self.emit(format!(
                        "{{\"t\":{},\"kind\":\"clash\",\"x\":{},\"y\":{},\"dx\":{},\"dy\":{},\"type\":\"clash\"}}",
                        t, f2(point[0]), f2(point[1]), f2(normal[0]), f2(normal[1])
                    ));
                }
                continue;
            }
            if self.exec[att].phase() == Phase::Active || self.whiff_pending[att] {
                self.exec[att].notify_contact();
                self.contact_seen[att] = true;
            }
            self.cooldown[att] = SWING_COOLDOWN;
            for ev in evs {
                let t = self.tick;
                match ev {
                    CombatEvent::Hit(h) => {
                        self.hitstop = self.hitstop.max(h.hitstop_ticks);
                        self.resolve_threat(att, Resolution::Hit);
                        self.dealt_w[att] += h.damage.max(0.0) * zone_weight(h.part);
                        self.quality_sum[att] += self.robots[att].strike_quality;
                        self.hits[att] += 1;
                        self.hit_list.push((t, att as u8));
                        if is_attack(att_id) {
                            self.hit_mask[att] |= 1 << att_id as u32;
                        }
                        if staggered || self.rules[def].is_open() {
                            self.grace[att].punish_hits += 1;
                        }
                        if self.exec[def].phase() == Phase::Recovery {
                            self.grace[def].recoveries_hit += 1;
                        }
                        self.last_hit = Some(t);
                        self.stall_acc += fitness::stall_units(t - self.stretch_start);
                        self.stretch_start = t;
                        for th in self.threats.iter_mut().rev() {
                            if th.attacker == att && matches!(th.resolution, Some(Resolution::Parried | Resolution::Blocked | Resolution::Dodged)) && t.saturating_sub(th.end_tick) <= 30 {
                                th.followup_hit = true;
                                break;
                            }
                        }
                        if total > 1.0 + 1e-4 {
                            self.emit(format!(
                                "{{\"t\":{},\"kind\":\"punish\",\"who\":{},\"victim\":{},\"name\":\"{}\",\"mult\":{}}}",
                                t, att, def, att_id.name(), f2(total)
                            ));
                        }
                        if def_wa {
                            self.emit(format!("{{\"t\":{},\"kind\":\"counter\",\"who\":{},\"name\":\"{}\"}}", t, att, att_id.name()));
                        }
                        self.emit(format!(
                            "{{\"t\":{},\"kind\":\"hit\",\"victim\":{},\"part\":\"{}\",\"x\":{},\"y\":{},\"dx\":{},\"dy\":{},\"impact\":{},\"zone\":{},\"dmg\":{}}}",
                            t, def, h.part.name(), f2(h.point[0]), f2(h.point[1]), f2(h.normal[0]), f2(h.normal[1]),
                            f2(h.impact), f2(h.zone), f2(h.damage)
                        ));
                        self.emit(format!("{{\"t\":{},\"kind\":\"hitstop\",\"ticks\":{}}}", t, h.hitstop_ticks));
                    }
                    CombatEvent::Clash { kind, point, normal } => {
                        // A Parry that meets a blade is a parry, whatever the closing speed; the blade lags the executor
                        // phases, so the early Recovery still counts. Same for every fighter.
                        let parrying = self.exec[def].move_id() == MoveId::Parry && crate::skillbot::is_attack(self.exec[att].move_id())
                            && (matches!(self.exec[def].phase(), Phase::Windup | Phase::Active)
                                || (self.exec[def].phase() == Phase::Recovery && self.exec[def].progress() < 0.5));
                        if kind != ClashKind::Clash || parrying {
                            self.resolve_threat(att, Resolution::Parried);
                            if parrying {
                                self.rules[def].on_parry_success();
                                self.grace[def].parry_success += 1;
                                self.contact_seen[def] = true;
                                self.exec[def].notify_contact();
                                self.emit(format!("{{\"t\":{},\"kind\":\"riposte_open\",\"who\":{}}}", t, def));
                            }
                        }
                        self.emit(format!(
                            "{{\"t\":{},\"kind\":\"clash\",\"x\":{},\"y\":{},\"dx\":{},\"dy\":{},\"type\":\"{}\"}}",
                            t, f2(point[0]), f2(point[1]), f2(normal[0]), f2(normal[1]), kind.log_name()
                        ));
                    }
                    CombatEvent::Block { raw, point, .. } => {
                        self.resolve_threat(att, Resolution::Blocked);
                        self.emit(format!(
                            "{{\"t\":{},\"kind\":\"block\",\"x\":{},\"y\":{},\"impact\":{}}}",
                            t, f2(point[0]), f2(point[1]), f2(raw)
                        ));
                    }
                }
            }
        }
    }

    /// Mutual stagger is a state nobody can punish, so a fighter cannot be broken while the opponent is already
    /// down: the blow leaves them on the brink instead. If both break on the same tick the steadier one (more
    /// energy) holds. The rule is the same for both sides.
    fn break_mutual_stagger(&mut self) {
        let (j0, j1) = (self.robots[0].just_staggered(), self.robots[1].just_staggered());
        if j0 && j1 {
            let (e0, e1) = (self.robots[0].energy, self.robots[1].energy);
            if e0 > e1 {
                self.robots[0].cancel_stagger(55.0);
            } else if e1 > e0 {
                self.robots[1].cancel_stagger(55.0);
            }
            return;
        }
        for i in 0..2 {
            if self.robots[i].just_staggered() && self.robots[1 - i].state() == State::Staggered {
                self.robots[i].cancel_stagger(55.0);
            }
        }
    }

    fn update_threats_post(&mut self) {
        for k in 0..self.threats.len() {
            let t = &self.threats[k];
            if t.resolution.is_some() {
                continue;
            }
            if t.quiet >= 8 || self.tick.saturating_sub(t.start_tick) > 60 {
                self.expire(k);
            }
        }
    }

    fn expire(&mut self, k: usize) {
        let t = &self.threats[k];
        let dodged = is_evasion(t.would_hit, t.moved, t.flexed);
        let (who, tick) = (t.defender, self.tick);
        self.threats[k].resolution = Some(if dodged { Resolution::Dodged } else { Resolution::Miss });
        self.threats[k].end_tick = tick;
        if dodged {
            self.emit(format!("{{\"t\":{},\"kind\":\"dodge\",\"who\":{}}}", tick, who));
            let (att, mv, bs) = (self.threats[k].attacker, self.threats[k].att_move, self.threats[k].def_backstep);
            if mv == MoveId::Thrust && bs {
                let (r, st) = (&mut self.robots[att], &mut self.rules[att]);
                rules::backstep_counter(r, st);
                self.emit(format!("{{\"t\":{},\"kind\":\"counter\",\"who\":{},\"name\":\"Thrust\"}}", tick, who));
            }
        }
    }

    fn finish_threats(&mut self) {
        for k in 0..self.threats.len() {
            if self.threats[k].resolution.is_none() {
                self.expire(k);
            }
        }
    }

    fn track_idle_guard(&mut self) {
        for i in 0..2 {
            let recent = self.last_live[i].map_or(false, |t| self.tick.saturating_sub(t) <= IDLE_GUARD_WINDOW);
            let active = self.threats.iter().any(|t| t.defender == i && t.resolution.is_none());
            if self.robots[i].guard_raised() && !recent && !active {
                self.idle_guard_ticks[i] += 1;
            }
        }
    }

    fn track_passivity(&mut self) {
        let far = (self.robots[0].x - self.robots[1].x).abs() > 3.5;
        let quiet = self.last_hit.map_or(true, |t| self.tick.saturating_sub(t) > 240);
        if far && quiet {
            self.passive_ticks += 1;
        }
    }

    fn track_states(&mut self) {
        for i in 0..2 {
            let st = self.robots[i].state();
            let t = self.tick;
            if st != self.prev_state[i] {
                match st {
                    State::Staggered => self.emit(format!("{{\"t\":{},\"kind\":\"stagger\",\"who\":{}}}", t, i)),
                    State::Fallen => self.emit(format!("{{\"t\":{},\"kind\":\"fall\",\"who\":{}}}", t, i)),
                    State::Ko => self.emit(format!("{{\"t\":{},\"kind\":\"ko\",\"who\":{}}}", t, i)),
                    State::Idle => {}
                }
                self.prev_state[i] = st;
            }
            for p in Part::ALL {
                let d = self.robots[i].is_destroyed(p);
                if d && !self.prev_destroyed[i][p as usize] {
                    self.emit(format!(
                        "{{\"t\":{},\"kind\":\"destroyed\",\"who\":{},\"part\":\"{}\"}}",
                        t,
                        i,
                        p.name()
                    ));
                }
                self.prev_destroyed[i][p as usize] = d;
            }
        }
    }

    pub fn result(&self) -> MatchResult {
        let ko = [self.robots[0].is_ko(), self.robots[1].is_ko()];
        // A death decides the fight; a mutual death or the emergency cap is a draw. Never a timeout win.
        let outcome = match ko {
            [true, false] => Outcome::WinB,
            [false, true] => Outcome::WinA,
            _ => Outcome::Draw,
        };
        let mut defence = [DefenceTally::default(); 2];
        let mut outs: [Vec<ThreatOutcome>; 2] = [Vec::new(), Vec::new()];
        for t in &self.threats {
            let d = &mut defence[t.defender];
            let res = t.resolution.unwrap_or(Resolution::Miss);
            match res {
                Resolution::Hit => d.hit += 1,
                Resolution::Parried => d.parry += 1,
                Resolution::Blocked => d.block += 1,
                Resolution::Dodged => d.dodge += 1,
                Resolution::Miss => d.miss += 1,
            }
            outs[t.defender].push(ThreatOutcome { resolution: res, i_potential: t.potential, followup_hit: t.followup_hit });
        }
        let credit = [fitness::defence_credit(&outs[0]), fitness::defence_credit(&outs[1])];
        let total = |i: usize| -> f32 { Part::ALL.iter().map(|p| self.robots[i].max_hp(*p)).sum() };
        let dealt = [(self.dealt_w[0] / total(1)).min(1.0), (self.dealt_w[1] / total(0)).min(1.0)];
        let q = |i: usize| if self.hits[i] > 0 { self.quality_sum[i] / self.hits[i] as f32 } else { 0.0 };
        let n = self.tick.max(1) as f32;
        MatchResult {
            outcome,
            ticks: self.tick,
            defence,
            idle_guard_share: [self.idle_guard_ticks[0] as f32 / n, self.idle_guard_ticks[1] as f32 / n],
            defence_credit: credit,
            ko,
            dmg_dealt_norm: dealt,
            passivity: self.passive_ticks as f32 / n,
            chain_quality: [q(0), q(1)],
            hits: self.hits,
            jerk: {
                let d = self.adv_ticks.max(1) as f32;
                [self.jerk_sum[0] / d, self.jerk_sum[1] / d]
            },
            capped: self.capped,
            stall: self.stall_acc + fitness::stall_units(self.tick.saturating_sub(self.stretch_start)),
            swings: [0, 1].map(|i| self.starts[i].iter().filter(|&&id| is_attack(id)).count() as u32),
            punish_hits: [self.grace[0].punish_hits, self.grace[1].punish_hits],
            open_windows: self.open_windows,
            landed_moves: [self.hit_mask[0].count_ones(), self.hit_mask[1].count_ones()],
            gap_mean: (self.gap_total / self.adv_ticks.max(1) as f64) as f32,
            engaged_gap: if self.engaged_ticks > 0 { (self.engaged_total / self.engaged_ticks as f64) as f32 } else { 4.0 },
            combo_mean: [combos(&self.hit_list, 0).0, combos(&self.hit_list, 1).0],
            combo_max: [combos(&self.hit_list, 0).1, combos(&self.hit_list, 1).1],
        }
    }

    pub fn run_to_end(&mut self) -> MatchResult {
        while !self.is_over() {
            self.step();
        }
        self.ended = true;
        self.finish_threats();
        self.result()
    }
}

/// Whether a swing moving from `prev` to `cur` meets any of the frozen defender capsules.
pub fn would_connect(prev: Capsule, cur: Capsule, snapshot: &[Capsule]) -> bool {
    snapshot.iter().any(|c| crate::combat::swept_hit(prev, cur, *c).is_some())
}

/// A dodge is an evasion only if the swing would have connected and the defender really moved or flexed.
pub fn is_evasion(would_hit: bool, moved: f32, flexed: f32) -> bool {
    would_hit && (moved >= 0.3 || flexed >= 20f32.to_radians())
}
