//! Attack definitions: hero moves and monster species movesets (data tables).
use crate::status::Status;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Hero,
    Minion,
    Rogue,
    Warrior,
    Mage,
}

impl Species {
    pub const ALL: [Species; 5] = [Species::Hero, Species::Minion, Species::Rogue, Species::Warrior, Species::Mage];
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Species::Hero => "hero",
            Species::Minion => "minion",
            Species::Rogue => "rogue",
            Species::Warrior => "warrior",
            Species::Mage => "mage",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    Grass,
    Elite,
    Boss,
}

impl Rank {
    pub const ALL: [Rank; 3] = [Rank::Grass, Rank::Elite, Rank::Boss];
    pub fn as_str(self) -> &'static str {
        match self {
            Rank::Grass => "grass",
            Rank::Elite => "elite",
            Rank::Boss => "boss",
        }
    }
}

pub const ELITE_DAMAGE_MULT: f32 = 1.15;
pub const BOSS_DAMAGE_MULT: f32 = 1.2;

/// One status applied by a hit. `chance` is always 1.0 (deterministic). For Knock `magnitude` is the
/// distance in tiles (duration comes from the combat tuning). `vulnerable_only` effects apply only
/// when the victim is Blocking or in Recovery.
#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    pub status: Status,
    pub chance: f32,
    pub duration: u32,
    pub magnitude: f32,
    pub vulnerable_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    pub name: &'static str,
    pub windup: u32,
    pub active: u32,
    pub recovery: u32,
    pub damage: f32,
    pub range: f32,
    pub half_arc: f32,
    pub stamina_cost: f32,
    /// Tiles moved forward during the active frames.
    pub lunge: f32,
    pub effects: Vec<Effect>,
}

fn fx(status: Status, duration: u32) -> Effect {
    Effect { status, chance: 1.0, duration, magnitude: 1.0, vulnerable_only: false }
}
fn knock(dist: f32) -> Effect {
    Effect { status: Status::Knock, chance: 1.0, duration: 0, magnitude: dist, vulnerable_only: false }
}
fn deg(d: f32) -> f32 {
    d * PI / 180.0
}
#[allow(clippy::too_many_arguments)]
fn mv(name: &'static str, w: u32, a: u32, r: u32, damage: f32, range: f32, half_arc_deg: f32, cost: f32, lunge: f32, effects: Vec<Effect>) -> Move {
    Move { name, windup: w, active: a, recovery: r, damage, range, half_arc: deg(half_arc_deg), stamina_cost: cost, lunge, effects }
}

fn base(species: Species) -> Vec<Move> {
    match species {
        Species::Hero => vec![
            mv("Light", 4, 3, 6, 8.0, 1.2, 45.0, 5.0, 0.0, vec![fx(Status::Stagger, 8)]),
            mv(
                "Heavy",
                12,
                4,
                14,
                18.0,
                1.3,
                45.0,
                15.0,
                0.0,
                vec![knock(2.0), Effect { vulnerable_only: true, ..fx(Status::Stun, 20) }],
            ),
            mv("Lunge", 8, 3, 12, 10.0, 1.6, 30.0, 12.0, 2.0, vec![fx(Status::Slowed, 90), fx(Status::Stagger, 6)]),
        ],
        Species::Minion => vec![
            mv("Swipe", 5, 3, 7, 6.0, 1.1, 45.0, 5.0, 0.0, vec![fx(Status::Stagger, 8)]),
            mv("Bite", 8, 3, 10, 9.0, 1.0, 20.0, 10.0, 0.0, vec![fx(Status::Bleed, 150)]),
        ],
        Species::Rogue => vec![
            mv("Stab", 3, 2, 8, 6.0, 1.1, 40.0, 6.0, 0.0, vec![fx(Status::Bleed, 150)]),
            mv("Dash", 6, 3, 12, 7.0, 1.5, 30.0, 12.0, 2.0, vec![fx(Status::Slowed, 90)]),
        ],
        Species::Warrior => vec![
            mv("Club", 14, 4, 16, 16.0, 1.3, 45.0, 15.0, 0.0, vec![knock(2.5), fx(Status::Stun, 24)]),
            mv("Cleave", 8, 4, 12, 9.0, 1.3, 70.0, 10.0, 0.0, vec![fx(Status::Stagger, 10)]),
        ],
        Species::Mage => vec![
            mv("Hex", 10, 3, 12, 5.0, 2.0, 30.0, 12.0, 0.0, vec![fx(Status::Poison, 240), fx(Status::Confused, 90)]),
            mv("Zap", 6, 3, 10, 6.0, 1.6, 45.0, 8.0, 0.0, vec![fx(Status::Stagger, 8), fx(Status::Slowed, 60)]),
        ],
    }
}

fn elite_extra(species: Species) -> Option<Effect> {
    match species {
        Species::Hero => None,
        Species::Minion => Some(fx(Status::Slowed, 60)),
        Species::Rogue => Some(fx(Status::Poison, 120)),
        Species::Warrior => Some(fx(Status::Bleed, 90)),
        Species::Mage => Some(knock(1.0)),
    }
}

/// The moveset for a species at a rank. Hero ignores rank.
pub fn moves(species: Species, rank: Rank) -> Vec<Move> {
    let mut v = base(species);
    if species == Species::Hero {
        return v;
    }
    match rank {
        Rank::Grass => {}
        Rank::Elite => {
            let extra = elite_extra(species);
            for m in v.iter_mut() {
                m.damage *= ELITE_DAMAGE_MULT;
                if let Some(e) = &extra {
                    m.effects.push(e.clone());
                }
            }
        }
        Rank::Boss => {
            v.push(mv("Slam", 16, 4, 16, 20.0, 1.6, 60.0, 20.0, 0.0, vec![fx(Status::Stun, 30), knock(2.0)]));
            for m in v.iter_mut() {
                m.damage *= BOSS_DAMAGE_MULT;
            }
        }
    }
    v
}
