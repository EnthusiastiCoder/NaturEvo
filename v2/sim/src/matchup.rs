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
            Fighter::new(s[0].0, s[0].1, 0.0, hp[0], cfg.max_stamina),
            Fighter::new(s[1].0, s[1].1, PI, hp[1], cfg.max_stamina),
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
            log: FightLog::with_capacity(cfg.tick_hz, cfg.max_ticks as usize),
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
        self.step_with(ah, am);
    }

    /// Advance one tick with scripted actions (bypasses the nets).
    pub fn step_with(&mut self, hero: Action, monster: Action) {
        if self.finished {
            return;
        }
        let mut acts = [hero, monster];
        for a in acts.iter_mut() {
            a.fwd = clean(a.fwd);
            a.strafe = clean(a.strafe);
            a.turn = clean(a.turn);
        }
        // 1. advance timers; swings that become active this tick land a hit
        let mut hit = [false; 2];
        for i in 0..2 {
            let f = &mut self.fighters[i];
            if f.dodge_cooldown > 0 {
                f.dodge_cooldown -= 1;
            }
            match f.state {
                CombatState::Windup => {
                    f.timer -= 1;
                    if f.timer == 0 {
                        f.state = CombatState::Active;
                        f.timer = ACTIVE_TICKS;
                        hit[i] = true;
                    }
                }
                CombatState::Active => {
                    f.timer -= 1;
                    if f.timer == 0 {
                        f.state = CombatState::Recovery;
                        f.timer = RECOVERY_TICKS;
                    }
                }
                CombatState::Recovery => {
                    f.timer -= 1;
                    if f.timer == 0 {
                        f.state = CombatState::Idle;
                    }
                }
                CombatState::Dodging => {
                    f.timer -= 1;
                    if f.timer == 0 {
                        f.state = CombatState::Idle;
                    }
                }
                _ => {}
            }
        }
        // 2. decisions
        for i in 0..2 {
            let a = acts[i];
            let f = &mut self.fighters[i];
            if matches!(f.state, CombatState::Idle | CombatState::Blocking) {
                if a.dodge && f.dodge_cooldown == 0 && f.stamina >= DODGE_STAMINA_COST {
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
