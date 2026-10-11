//! Duel loop: two robots, brains, combat per tick, threat tracking, anti-stalemate bleed, outcome.
use crate::arena::Arena;
use crate::combat::{resolve_tick, CombatEvent, ClashKind};
use crate::fitness::{self, ThreatOutcome, zone_weight};
use crate::config::{DECISION_TICKS, MAX_TICKS, PHYS_DT};
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
    pub end_tick: u32,
    pub followup_hit: bool,
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
    pub ko: [bool; 2],
    /// Zone-weighted damage dealt by each fighter, over the opponent's total hp (0..1).
    pub dmg_dealt_norm: [f32; 2],
    /// Share of the match spent far apart with no hit exchanged recently.
    pub passivity: f32,
    /// Mean strike quality of each fighter's landed hits.
    pub chain_quality: [f32; 2],
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
    dealt_w: [f32; 2],
    quality_sum: [f32; 2],
    hits: [u32; 2],
    last_hit: Option<u32>,
    passive_ticks: u32,
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
            dealt_w: [0.0; 2],
            quality_sum: [0.0; 2],
            hits: [0; 2],
            last_hit: None,
            passive_ticks: 0,
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

    pub fn enable_log(&mut self) {
        if self.log.is_none() {
            self.log = Some(FightLog::default());
        }
    }

    pub fn is_over(&self) -> bool {
        self.ended || self.tick >= MAX_TICKS || self.robots.iter().any(|r| r.is_ko())
    }

    fn emit(&mut self, e: String) {
        if let Some(l) = self.log.as_mut() {
            l.events.push(e);
        }
    }

    fn decide(&mut self, i: usize) -> Action {
        let (me, opp) = (&self.robots[i], &self.robots[1 - i]);
        match &mut self.ctl[i] {
            Ctl::Dummy => scripted::dummy(),
            Ctl::Fixed(a) => *a,
            Ctl::Scripted(s) => s.act(me, opp, PHYS_DT),
            Ctl::Net(net, held) => {
                if self.tick % DECISION_TICKS == 0 {
                    let x = observe(me, opp, &self.arena);
                    let o = net.forward(&x);
                    let mut t = [0.0f32; 12];
                    t.copy_from_slice(&o[..12]);
                    *held = Action { targets: t, drive: o[12] };
                }
                *held
            }
        }
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
        } else {
            self.advance_fighters();
            self.update_threats_pre();
            self.run_combat();
        }
        self.update_threats_post();
        self.bleed();
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
        if self.tick >= MAX_TICKS || self.robots.iter().any(|r| r.is_ko()) {
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
        for i in 0..2 {
            let act = self.decide(i);
            let old_x = self.robots[i].x;
            let r = &mut self.robots[i];
            r.apply_targets(&act.targets, PHYS_DT);
            r.apply_drive(act.drive, PHYS_DT, self.arena.width);
            let nx = self.robots[i].x;
            if nx != old_x && self.blocked_by_obstacle(nx) && !self.blocked_by_obstacle(old_x) {
                self.robots[i].x = old_x;
                self.robots[i].fk();
            }
        }
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
            start_x: d.x,
            start_angles: a,
            moved: 0.0,
            flexed: 0.0,
            quiet: 0,
        };
        self.next_threat += 1;
        self.threats.push(t);
        self.threats.len() - 1
    }

    fn resolve_threat(&mut self, att: usize, res: Resolution) {
        let k = match self.threats.iter().position(|t| t.attacker == att && t.resolution.is_none()) {
            Some(k) => k,
            None => {
                let k = self.create_threat(att, 0.0);
                k
            }
        };
        self.threats[k].resolution = Some(res);
        self.threats[k].end_tick = self.tick;
    }

    fn run_combat(&mut self) {
        for att in 0..2 {
            if self.cooldown[att] > 0 {
                self.cooldown[att] -= 1;
                continue;
            }
            let def = 1 - att;
            let evs = {
                let (l, r) = self.robots.split_at_mut(1);
                let (a, b) = if att == 0 { (&mut l[0], &mut r[0]) } else { (&mut r[0], &mut l[0]) };
                resolve_tick(a, b, &self.arena)
            };
            if evs.is_empty() {
                continue;
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
                        self.last_hit = Some(t);
                        for th in self.threats.iter_mut().rev() {
                            if th.attacker == att && matches!(th.resolution, Some(Resolution::Parried | Resolution::Blocked | Resolution::Dodged)) && t.saturating_sub(th.end_tick) <= 30 {
                                th.followup_hit = true;
                                break;
                            }
                        }
                        self.emit(format!(
                            "{{\"t\":{},\"kind\":\"hit\",\"victim\":{},\"part\":\"{}\",\"x\":{},\"y\":{},\"dx\":{},\"dy\":{},\"impact\":{},\"zone\":{},\"dmg\":{}}}",
                            t, def, h.part.name(), f2(h.point[0]), f2(h.point[1]), f2(h.normal[0]), f2(h.normal[1]),
                            f2(h.impact), f2(h.zone), f2(h.damage)
                        ));
                        self.emit(format!("{{\"t\":{},\"kind\":\"hitstop\",\"ticks\":{}}}", t, h.hitstop_ticks));
                    }
                    CombatEvent::Clash { kind, point, normal } => {
                        if kind != ClashKind::Clash {
                            self.resolve_threat(att, Resolution::Parried);
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
        let dodged = t.moved >= 0.3 || t.flexed >= 20f32.to_radians();
        let (who, tick) = (t.defender, self.tick);
        self.threats[k].resolution = Some(if dodged { Resolution::Dodged } else { Resolution::Miss });
        self.threats[k].end_tick = tick;
        if dodged {
            self.emit(format!("{{\"t\":{},\"kind\":\"dodge\",\"who\":{}}}", tick, who));
        }
    }

    fn finish_threats(&mut self) {
        for k in 0..self.threats.len() {
            if self.threats[k].resolution.is_none() {
                self.expire(k);
            }
        }
    }

    fn bleed(&mut self) {
        if self.tick < BLEED_START {
            return;
        }
        for r in self.robots.iter_mut() {
            let hp = r.hp(Part::Torso);
            r.set_hp(Part::Torso, hp - BLEED_PER_SEC * PHYS_DT);
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

    fn hp_fraction(&self, i: usize) -> f32 {
        let r = &self.robots[i];
        let (mut s, mut m) = (0.0, 0.0);
        for p in Part::ALL {
            s += r.hp(p).max(0.0);
            m += r.max_hp(p);
        }
        s / m
    }

    pub fn result(&self) -> MatchResult {
        let ko = [self.robots[0].is_ko(), self.robots[1].is_ko()];
        let outcome = match ko {
            [true, false] => Outcome::WinB,
            [false, true] => Outcome::WinA,
            [true, true] => Outcome::Draw,
            _ => {
                let (a, b) = (self.hp_fraction(0), self.hp_fraction(1));
                if a - b >= 0.05 {
                    Outcome::WinA
                } else if b - a >= 0.05 {
                    Outcome::WinB
                } else {
                    Outcome::Draw
                }
            }
        };
        let mut defence = [DefenceTally::default(); 2];
        let mut credit = [0.0f32; 2];
        let mut counted = [0u32; 2];
        for t in &self.threats {
            let d = &mut defence[t.defender];
            let frac = (t.potential / IMPACT_REF).min(1.0);
            let c = match t.resolution {
                Some(Resolution::Hit) => {
                    d.hit += 1;
                    0.0
                }
                Some(Resolution::Parried) => {
                    d.parry += 1;
                    frac
                }
                Some(Resolution::Blocked) => {
                    d.block += 1;
                    0.7
                }
                Some(Resolution::Dodged) => {
                    d.dodge += 1;
                    0.7
                }
                Some(Resolution::Miss) | None => {
                    d.miss += 1;
                    0.0
                }
            };
            if c > 0.0 && counted[t.defender] < 6 {
                counted[t.defender] += 1;
                credit[t.defender] += c;
            }
        }
        let n = self.tick.max(1) as f32;
        MatchResult {
            outcome,
            ticks: self.tick,
            defence,
            idle_guard_share: [self.idle_guard_ticks[0] as f32 / n, self.idle_guard_ticks[1] as f32 / n],
            defence_credit: credit,
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
