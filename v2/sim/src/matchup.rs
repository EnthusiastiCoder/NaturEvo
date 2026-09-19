//! A single real-time 1v1 duel: physics, combat state machine, status effects, logging.
//! Every gameplay constant lives in `config::CombatTuning` or the move tables in `attacks.rs`.
use crate::attacks::{moves, Move, Rank, Species};
use crate::config::ArenaConfig;
pub use crate::config::BLOCK_HALF_ARC;
use crate::fighter::{Action, CombatState, Fighter, Role};
use crate::genome::Genome;
use crate::log::{Event, FightLog, Frame};
use crate::net::Net;
use crate::sense::{observe, sees};
use crate::status::{confusion_hash, Status};
use crate::world::Arena;
use std::f32::consts::PI;

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

/// Who fights: starting hp, species and rank per side (index 0 hero, 1 monster).
#[derive(Clone, Copy, Debug)]
pub struct Setup {
    pub hp: [f32; 2],
    pub species: [Species; 2],
    pub rank: [Rank; 2],
}

#[derive(Clone, Debug)]
pub struct Match {
    cfg: ArenaConfig,
    arena: Arena,
    fighters: [Fighter; 2],
    moveset: [Vec<Move>; 2],
    nets: [Net; 2],
    tick: u32,
    finished: bool,
    outcome: Outcome,
    damage_dealt: [f32; 2],
    log: FightLog,
    events: Vec<Event>,
    seed: u64,
}

/// Pre-hit view of a fighter, so simultaneous hits resolve against the same state.
#[derive(Clone, Copy)]
struct Snap {
    state: CombatState,
    x: f32,
    y: f32,
    facing: f32,
    stamina: f32,
    block_age: u32,
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
        Match::new_with_hp(arena, hero, monster, seed, cfg, [cfg.max_hp, cfg.max_hp])
    }

    /// Like `new` but with explicit starting hp per side (campaign handicaps). Arena rules are unchanged.
    pub fn new_with_hp(arena: Arena, hero: Genome, monster: Genome, seed: u64, cfg: &ArenaConfig, hp: [f32; 2]) -> Match {
        let setup = Setup { hp, species: [Species::Hero, Species::Minion], rank: [Rank::Grass, Rank::Grass] };
        Match::new_with_setup(arena, hero, monster, seed, cfg, setup)
    }

    pub fn new_with_setup(arena: Arena, hero: Genome, monster: Genome, seed: u64, cfg: &ArenaConfig, setup: Setup) -> Match {
        let s = arena.spawns();
        let fighters = [
            Fighter::new(s[0].0, s[0].1, 0.0, setup.hp[0], cfg.max_stamina),
            Fighter::new(s[1].0, s[1].1, PI, setup.hp[1], cfg.max_stamina),
        ];
        let mut log = FightLog::with_capacity(cfg.tick_hz, cfg.max_ticks as usize);
        log.meta = Some((setup.species, setup.rank));
        Match {
            cfg: cfg.clone(),
            arena,
            fighters,
            moveset: [moves(setup.species[0], setup.rank[0]), moves(setup.species[1], setup.rank[1])],
            nets: [Net::from_genome(&hero), Net::from_genome(&monster)],
            tick: 0,
            finished: false,
            outcome: Outcome::Draw,
            damage_dealt: [0.0; 2],
            log,
            events: Vec::new(),
            seed,
        }
    }

    pub fn cfg(&self) -> &ArenaConfig {
        &self.cfg
    }
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
    pub fn fighter(&self, r: Role) -> &Fighter {
        &self.fighters[r.index()]
    }
    pub fn tick(&self) -> u32 {
        self.tick
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }
    pub fn hp(&self, r: Role) -> f32 {
        self.fighters[r.index()].hp
    }
    pub fn stamina(&self, r: Role) -> f32 {
        self.fighters[r.index()].stamina
    }
    pub fn pos(&self, r: Role) -> (f32, f32) {
        let f = &self.fighters[r.index()];
        (f.x, f.y)
    }
    pub fn damage_dealt(&self) -> [f32; 2] {
        self.damage_dealt
    }
    pub fn log(&self) -> &FightLog {
        &self.log
    }
    /// Remaining ticks per status, indexed by `Status::idx`.
    pub fn fx(&self, r: Role) -> [u32; 7] {
        self.fighters[r.index()].status.remaining
    }
    pub fn stacks(&self, r: Role) -> [u8; 7] {
        self.fighters[r.index()].status.stacks
    }

    // ---- test hooks ----

    /// Face each other, `dist` apart, on open ground in the arena centre.
    pub fn place_for_duel(&mut self, dist: f32) {
        let cx = self.cfg.arena_w / 2.0;
        let cy = self.cfg.arena_h / 2.0 + 0.5;
        self.fighters[0].x = cx - dist / 2.0;
        self.fighters[0].y = cy;
        self.fighters[0].facing = 0.0;
        self.fighters[1].x = cx + dist / 2.0;
        self.fighters[1].y = cy;
        self.fighters[1].facing = PI;
    }

    pub fn face_away(&mut self, r: Role) {
        let o = self.fighters[r.other().index()].clone();
        let f = &mut self.fighters[r.index()];
        f.facing = wrap_angle((o.y - f.y).atan2(o.x - f.x) + PI);
    }

    pub fn set_stamina(&mut self, r: Role, s: f32) {
        self.fighters[r.index()].stamina = s;
    }
    pub fn set_hp(&mut self, r: Role, hp: f32) {
        self.fighters[r.index()].hp = hp;
    }
    pub fn set_pos(&mut self, r: Role, x: f32, y: f32) {
        let f = &mut self.fighters[r.index()];
        f.x = x;
        f.y = y;
    }
    pub fn set_facing(&mut self, r: Role, facing: f32) {
        self.fighters[r.index()].facing = wrap_angle(facing);
    }

    // ---- statuses ----

    /// Apply a status. Ignored for dead or dodging fighters and while the relevant immunity holds.
    /// For Knock, `mag` is the distance in tiles and `dur` the ticks it takes.
    pub fn apply_status(&mut self, role: Role, st: Status, dur: u32, mag: f32, source: Role) {
        self.apply_inner(role.index(), st, dur, mag, source.index());
    }

    fn apply_inner(&mut self, j: usize, st: Status, dur: u32, mag: f32, src: usize) {
        if dur == 0 {
            return;
        }
        let state = self.fighters[j].state;
        if matches!(state, CombatState::Dead | CombatState::Dodging) {
            return;
        }
        let now = self.tick;
        let t = &self.cfg.combat;
        let k = st.idx();
        match st {
            Status::Stagger => {
                let s = &mut self.fighters[j].status;
                if s.stagger_imm > 0 || matches!(state, CombatState::Active | CombatState::GuardRaise) {
                    return;
                }
                s.remaining[k] = s.remaining[k].max(dur);
                s.stacks[k] = 1;
            }
            Status::Stun => {
                let s = &mut self.fighters[j].status;
                if s.remaining[k] > 0 || s.stun_imm > 0 {
                    return;
                }
                let d = s.stun_duration(dur, now, t);
                s.remaining[k] = d;
                s.stacks[k] = 1;
                s.stun_hist.push(now);
            }
            Status::Knock => {
                if self.fighters[j].status.knock_imm > 0 || self.fighters[j].status.remaining[k] > 0 {
                    return;
                }
                let (dx, dy) = (self.fighters[j].x - self.fighters[src].x, self.fighters[j].y - self.fighters[src].y);
                let len = (dx * dx + dy * dy).sqrt();
                let (ux, uy) = if len > 1e-6 { (dx / len, dy / len) } else { (1.0, 0.0) };
                let per = mag.max(0.0) / dur as f32;
                let f = &mut self.fighters[j];
                f.status.knock_step = (ux * per, uy * per);
                f.status.remaining[k] = dur;
                f.status.stacks[k] = 1;
                f.knock_short = 0.0;
            }
            Status::Confused => {
                let s = &mut self.fighters[j].status;
                s.remaining[k] = s.remaining[k].max(dur);
                s.stacks[k] = 1;
            }
            Status::Slowed => {
                let s = &mut self.fighters[j].status;
                s.stacks[k] = (s.stacks[k] + 1).min(t.slowed_max_stacks);
                s.remaining[k] = s.remaining[k].max(dur);
            }
            Status::Bleed => {
                let s = &mut self.fighters[j].status;
                if s.remaining[k] == 0 {
                    s.bleed_timer = t.dot_interval;
                    s.stacks[k] = 0;
                }
                s.stacks[k] = (s.stacks[k] + 1).min(t.bleed_max_stacks);
                s.remaining[k] = s.remaining[k].max(dur);
            }
            Status::Poison => {
                let s = &mut self.fighters[j].status;
                if s.remaining[k] == 0 {
                    s.poison_timer = t.dot_interval;
                }
                s.remaining[k] = (s.remaining[k] + dur).min(t.poison_cap);
                s.stacks[k] = 1;
            }
        }
        self.fighters[j].status.source[k] = src;
        self.sync_state(j);
    }

    /// Make the combat state agree with the active statuses (stun over knock over stagger).
    fn sync_state(&mut self, j: usize) {
        let f = &mut self.fighters[j];
        if matches!(f.state, CombatState::Dead | CombatState::Dodging) {
            return;
        }
        let s = &f.status;
        let want = if s.active(Status::Stun) {
            Some(CombatState::Stunned)
        } else if s.active(Status::Knock) {
            Some(CombatState::Knocked)
        } else if s.active(Status::Stagger) && !matches!(f.state, CombatState::Active | CombatState::GuardRaise) {
            Some(CombatState::Staggered)
        } else {
            None
        };
        match want {
            Some(ns) => {
                if f.state != ns {
                    f.state = ns;
                    f.timer = 0;
                }
            }
            None => {
                if matches!(f.state, CombatState::Staggered | CombatState::Stunned | CombatState::Knocked) {
                    f.state = CombatState::Idle;
                    f.timer = 0;
                }
            }
        }
    }

    fn damage(&mut self, j: usize, amount: f32, src: usize, floor: f32) -> f32 {
        let f = &mut self.fighters[j];
        let loss = amount.min((f.hp - floor).max(0.0));
        if loss > 0.0 {
            f.hp -= loss;
            self.damage_dealt[src] += loss;
        }
        if f.hp <= 0.0 {
            f.hp = 0.0;
            f.state = CombatState::Dead;
        }
        loss
    }

    // ---- simulation ----

    /// Advance one tick using the nets.
    pub fn step(&mut self) {
        if self.finished {
            return;
        }
        let oh = observe(self, Role::Hero);
        let om = observe(self, Role::Monster);
        let ah = self.nets[0].forward(&oh);
        let am = self.nets[1].forward(&om);
        // Observation slot 37 is set exactly when the other fighter is seen.
        self.step_inner(ah, am, Some([oh[37] > 0.0, om[37] > 0.0]));
    }

    /// Advance one tick with scripted actions (bypasses the nets).
    pub fn step_with(&mut self, hero: Action, monster: Action) {
        self.step_inner(hero, monster, None);
    }

    fn step_inner(&mut self, hero: Action, monster: Action, seen: Option<[bool; 2]>) {
        if self.finished {
            return;
        }
        let mut acts = [hero, monster];
        for a in acts.iter_mut() {
            a.fwd = clean(a.fwd);
            a.strafe = clean(a.strafe);
            a.turn = clean(a.turn);
        }
        let mut hit = [false; 2];
        for i in 0..2 {
            hit[i] = self.advance_timers(i, &acts[i]);
        }
        for i in 0..2 {
            self.decide(i, &acts[i]);
        }
        for i in 0..2 {
            self.move_fighter(i, acts[i]);
        }
        self.resolve_hits(hit);
        // stamina regen while idle
        let hz = self.cfg.tick_hz as f32;
        let t = self.cfg.combat;
        for f in self.fighters.iter_mut() {
            if f.state == CombatState::Idle {
                let mut regen = t.stamina_regen_per_sec / hz;
                if f.status.active(Status::Poison) {
                    regen *= t.poison_regen_mult;
                }
                f.stamina = (f.stamina + regen).min(self.cfg.max_stamina);
            }
        }
        // bookkeeping
        self.tick += 1;
        let swinging = |f: &Fighter| {
            if matches!(f.state, CombatState::Windup | CombatState::Active | CombatState::Recovery) {
                f.move_idx as i8
            } else {
                -1
            }
        };
        let seen = seen.unwrap_or_else(|| [sees(self, Role::Hero), sees(self, Role::Monster)]);
        let (f0, f1) = (&self.fighters[0], &self.fighters[1]);
        let frame = Frame {
            pos: [(f0.x, f0.y), (f1.x, f1.y)],
            facing: [f0.facing, f1.facing],
            hp: [f0.hp, f1.hp],
            stamina: [f0.stamina, f1.stamina],
            action: [f0.last_action, f1.last_action],
            state: [f0.state, f1.state],
            atk: [swinging(f0), swinging(f1)],
            fx: [f0.status.remaining, f1.status.remaining],
            seen,
            ev: std::mem::take(&mut self.events),
        };
        self.log.frames.push(frame);
        let hero_dead = self.fighters[0].state == CombatState::Dead;
        let mon_dead = self.fighters[1].state == CombatState::Dead;
        if hero_dead || mon_dead {
            self.finished = true;
            self.outcome = match (hero_dead, mon_dead) {
                (false, true) => Outcome::HeroWin,
                (true, false) => Outcome::MonsterWin,
                _ => Outcome::Draw,
            };
        } else if self.tick >= self.cfg.max_ticks {
            self.finished = true;
            self.outcome = Outcome::Draw;
        }
    }

    /// Phase 1: cooldowns, state timers and status timers. Returns whether a swing became active.
    fn advance_timers(&mut self, i: usize, act: &Action) -> bool {
        if self.fighters[i].state == CombatState::Dead {
            return false;
        }
        let mut hit = false;
        {
            let t = &self.cfg.combat;
            let f = &mut self.fighters[i];
            f.dodge_cooldown = f.dodge_cooldown.saturating_sub(1);
            f.status.tick_immunities();
            if f.state == CombatState::Blocking {
                f.block_age += 1;
            }
            let mv = &self.moveset[i][f.move_idx.min(self.moveset[i].len() - 1)];
            match f.state {
                CombatState::Windup => {
                    f.timer = f.timer.saturating_sub(1);
                    if f.timer == 0 {
                        f.state = CombatState::Active;
                        f.timer = mv.active;
                        hit = true;
                    }
                }
                CombatState::Active => {
                    f.timer = f.timer.saturating_sub(1);
                    if f.timer == 0 {
                        f.state = CombatState::Recovery;
                        f.timer = if f.swing_hit {
                            mv.recovery
                        } else {
                            (mv.recovery as f32 * t.whiff_recovery_mult).round() as u32
                        };
                    }
                }
                CombatState::Recovery | CombatState::Dodging | CombatState::GuardRecover => {
                    f.timer = f.timer.saturating_sub(1);
                    if f.timer == 0 {
                        f.state = CombatState::Idle;
                    }
                }
                CombatState::GuardRaise => {
                    f.timer = f.timer.saturating_sub(1);
                    if f.timer == 0 {
                        f.state = CombatState::Blocking;
                        f.block_age = 0;
                    }
                }
                _ => {}
            }
        }
        self.advance_statuses(i, act);
        self.sync_state(i);
        hit
    }

    fn advance_statuses(&mut self, i: usize, act: &Action) {
        // Knockback displacement.
        let k = Status::Knock.idx();
        if self.fighters[i].status.remaining[k] > 0 {
            let (sx, sy) = self.fighters[i].status.knock_step;
            let planned = (sx * sx + sy * sy).sqrt();
            let (x, y) = (self.fighters[i].x, self.fighters[i].y);
            let mut moved = 0.0;
            for (mx, my) in [(sx, sy), (sx, 0.0), (0.0, sy)] {
                if (mx != 0.0 || my != 0.0) && self.can_stand(i, x + mx, y + my) {
                    self.fighters[i].x = x + mx;
                    self.fighters[i].y = y + my;
                    moved = (mx * mx + my * my).sqrt();
                    break;
                }
            }
            self.fighters[i].knock_short += (planned - moved).max(0.0);
        }
        let t = self.cfg.combat;
        for st in Status::ALL {
            let k = st.idx();
            if self.fighters[i].status.remaining[k] == 0 {
                continue;
            }
            let src = self.fighters[i].status.source[k];
            match st {
                Status::Bleed | Status::Poison => {
                    let timer = if st == Status::Bleed { &mut self.fighters[i].status.bleed_timer } else { &mut self.fighters[i].status.poison_timer };
                    *timer = timer.saturating_sub(1);
                    if *timer == 0 {
                        *timer = t.dot_interval;
                        if st == Status::Bleed {
                            let mut d = t.dot_damage * self.fighters[i].status.stacks[k] as f32;
                            if act.fwd.abs() > t.bleed_move_threshold {
                                d *= t.bleed_move_mult;
                            }
                            let loss = self.damage(i, d, src, 0.0);
                            self.events.push(Event::Dot { who: i, kind: "bleed", dmg: loss });
                        } else {
                            let loss = self.damage(i, t.dot_damage, src, 1.0);
                            self.events.push(Event::Dot { who: i, kind: "poison", dmg: loss });
                        }
                    }
                }
                _ => {}
            }
            if self.fighters[i].state == CombatState::Dead {
                return;
            }
            let s = &mut self.fighters[i].status;
            s.remaining[k] -= 1;
            if s.remaining[k] == 0 {
                s.stacks[k] = 0;
                match st {
                    Status::Stagger => s.stagger_imm = t.stagger_immunity,
                    Status::Stun => s.stun_imm = t.stun_immunity,
                    Status::Knock => {
                        s.knock_imm = t.knock_immunity;
                        if self.fighters[i].knock_short > t.wall_slam_min_short {
                            let loss = self.damage(i, t.wall_slam_damage, src, 0.0);
                            self.events.push(Event::Slam { who: i, dmg: loss });
                            self.apply_inner(i, Status::Stagger, t.wall_slam_stagger, 1.0, src);
                        }
                        self.fighters[i].knock_short = 0.0;
                    }
                    _ => {}
                }
                if self.fighters[i].state == CombatState::Dead {
                    return;
                }
            }
        }
    }

    /// Phase 2: what each fighter chooses to do this tick.
    fn decide(&mut self, i: usize, a: &Action) {
        let t = &self.cfg.combat;
        let hz = self.cfg.tick_hz as f32;
        let f = &mut self.fighters[i];
        f.last_action = *a;
        let can_dodge = a.dodge && f.dodge_cooldown == 0 && f.stamina >= t.dodge_stamina_cost;
        match f.state {
            CombatState::Idle => {
                if can_dodge {
                    f.state = CombatState::Dodging;
                    f.timer = DODGE_TICKS;
                    f.stamina -= DODGE_STAMINA_COST;
                    f.dodge_cooldown = DODGE_COOLDOWN;
                } else if a.attack {
                    f.state = CombatState::Windup;
                    f.timer = WINDUP_TICKS;
                } else if a.block {
                    f.state = CombatState::Blocking;
                } else {
                    f.state = CombatState::Idle;
                }
            }
            f.last_action = a;
        }
        // 3. movement
        for i in 0..2 {
            self.move_fighter(i, acts[i]);
        }
        // 4. hits (resolved against pre-hit states, so trades are simultaneous)
        let mut dmg = [0.0f32; 2];
        let mut stam_cost = [0.0f32; 2];
        for i in 0..2 {
            if !hit[i] {
                continue;
            }
            let j = 1 - i;
            let (a, t) = (&self.fighters[i], &self.fighters[j]);
            if t.state == CombatState::Dead || t.state == CombatState::Dodging {
                continue;
            }
            let (dx, dy) = (t.x - a.x, t.y - a.y);
            if (dx * dx + dy * dy).sqrt() > ATTACK_RANGE {
                continue;
            }
            if wrap_angle(dy.atan2(dx) - a.facing).abs() > ATTACK_HALF_ARC {
                continue;
            }
            let mut d = DAMAGE;
            if t.state == CombatState::Blocking && t.stamina > 0.0 {
                let to_att = (-dy).atan2(-dx);
                if wrap_angle(to_att - t.facing).abs() <= BLOCK_HALF_ARC {
                    d *= 1.0 - BLOCK_REDUCTION;
                    stam_cost[j] += BLOCK_STAMINA_COST;
                }
            }
            dmg[j] += d;
        }
        for j in 0..2 {
            let f = &mut self.fighters[j];
            if stam_cost[j] > 0.0 {
                f.stamina = (f.stamina - stam_cost[j]).max(0.0);
            }
            if dmg[j] > 0.0 {
                let loss = dmg[j].min(f.hp);
                f.hp -= loss;
                self.damage_dealt[1 - j] += loss;
                if f.hp <= 0.0 {
                    f.hp = 0.0;
                    f.state = CombatState::Dead;
                }
            }
        }
        // 5. stamina regen while idle
        let regen = STAMINA_REGEN_PER_SEC / self.cfg.tick_hz as f32;
        for f in self.fighters.iter_mut() {
            if f.state == CombatState::Idle {
                f.stamina = (f.stamina + regen).min(self.cfg.max_stamina);
            }
        }
        // 6. bookkeeping
        self.tick += 1;
        self.log.frames.push(Frame {
            pos: [(self.fighters[0].x, self.fighters[0].y), (self.fighters[1].x, self.fighters[1].y)],
            facing: [self.fighters[0].facing, self.fighters[1].facing],
            hp: [self.fighters[0].hp, self.fighters[1].hp],
            stamina: [self.fighters[0].stamina, self.fighters[1].stamina],
            action: [self.fighters[0].last_action, self.fighters[1].last_action],
            state: [self.fighters[0].state, self.fighters[1].state],
        });
        let hero_dead = self.fighters[0].state == CombatState::Dead;
        let mon_dead = self.fighters[1].state == CombatState::Dead;
        if hero_dead || mon_dead {
            self.finished = true;
            self.outcome = match (hero_dead, mon_dead) {
                (false, true) => Outcome::HeroWin,
                (true, false) => Outcome::MonsterWin,
                _ => Outcome::Draw,
            };
        } else if self.tick >= self.cfg.max_ticks {
            self.finished = true;
            self.outcome = Outcome::Draw;
        }
    }

    fn move_fighter(&mut self, i: usize, a: Action) {
        let state = self.fighters[i].state;
        if state == CombatState::Dead {
            return;
        }
        let hz = self.cfg.tick_hz as f32;
        let f = &mut self.fighters[i];
        f.facing = wrap_angle(f.facing + a.turn * self.cfg.max_turn / hz);
        let factor = match state {
            CombatState::Windup | CombatState::Active | CombatState::Recovery => 0.3,
            CombatState::Blocking => 0.5,
            CombatState::Dodging => 1.5,
            _ => 1.0,
        };
        let len = (a.fwd * a.fwd + a.strafe * a.strafe).sqrt();
        if len <= 0.0 {
            return;
        }
        let scale = if len > 1.0 { 1.0 / len } else { 1.0 };
        let step = self.cfg.max_speed / hz * factor * scale;
        let (sin, cos) = f.facing.sin_cos();
        let dx = (cos * a.fwd - sin * a.strafe) * step;
        let dy = (sin * a.fwd + cos * a.strafe) * step;
        let (x, y) = (f.x, f.y);
        for (mx, my) in [(dx, dy), (dx, 0.0), (0.0, dy)] {
            if (mx != 0.0 || my != 0.0) && self.can_stand(i, x + mx, y + my) {
                self.fighters[i].x = x + mx;
                self.fighters[i].y = y + my;
                return;
            }
        }
    }

    fn can_stand(&self, i: usize, x: f32, y: f32) -> bool {
        let r = self.cfg.fighter_radius;
        if self.arena.circle_blocked(x, y, r) {
            return false;
        }
        let o = &self.fighters[1 - i];
        if o.state != CombatState::Dead {
            let me = &self.fighters[i];
            let new_d2 = (x - o.x).powi(2) + (y - o.y).powi(2);
            let old_d2 = (me.x - o.x).powi(2) + (me.y - o.y).powi(2);
            if new_d2 < 4.0 * r * r && new_d2 < old_d2 {
                return false;
            }
        }
        true
    }
}

/// Play a full match with the nets and return the result and log.
pub fn play(arena: Arena, hero: Genome, monster: Genome, seed: u64, cfg: &ArenaConfig) -> MatchResult {
    play_with_hp(arena, hero, monster, seed, cfg, [cfg.max_hp, cfg.max_hp])
}

/// `play` with explicit starting hp per side.
pub fn play_with_hp(arena: Arena, hero: Genome, monster: Genome, seed: u64, cfg: &ArenaConfig, hp: [f32; 2]) -> MatchResult {
    let mut m = Match::new_with_hp(arena, hero, monster, seed, cfg, hp);
    while !m.finished() {
        m.step();
    }
    MatchResult {
        outcome: m.outcome,
        ticks: m.tick,
        hero_hp: m.fighters[0].hp,
        monster_hp: m.fighters[1].hp,
        damage_dealt: m.damage_dealt,
        log: m.log,
    }
}
