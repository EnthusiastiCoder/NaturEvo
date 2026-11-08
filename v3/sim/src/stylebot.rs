//! Scripted style teachers: three archetypes built from the existing move vocabulary, each a function of what a
//! fighter can sense (distance, own and opponent move phase, opening flags), so a net can imitate them.
//! They fight through the same Executor as the net and obey the same rules as every fighter.
//! Press: advances, chains attacks from close range, rarely defends. Counter: holds distance, defends against
//! every telegraph, then ripostes and punishes. Duelist: keeps its optimal spacing, strikes singly with feints
//! and drag, steps out of reach of incoming swings.
use crate::moveset::{MoveId, MoveParams, Phase};
use crate::robot::{Robot, State};
use crate::rng::Rng;
use crate::sense::MoveInfo;
use crate::skillbot::{is_attack, Request};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleId {
    Press,
    Counter,
    Duelist,
}

impl StyleId {
    pub const ALL: [StyleId; 3] = [StyleId::Press, StyleId::Counter, StyleId::Duelist];
    pub fn name(self) -> &'static str {
        match self {
            StyleId::Press => "press",
            StyleId::Counter => "counter",
            StyleId::Duelist => "duelist",
        }
    }
    pub fn from_name(s: &str) -> Option<StyleId> {
        StyleId::ALL.iter().copied().find(|x| x.name() == s)
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

fn p(reach: f32, speed: f32, arc: f32, delay: f32) -> MoveParams {
    MoveParams { reach, speed, arc, delay }
}

const SWING_MAX: f32 = 2.2;
const STUCK_RANGE: f32 = 2.55;
const CLOSE: f32 = 1.0;

/// Per-style tuning: preferred spacing, reaction point and the like.
struct Tune {
    /// Walk toward the opponent beyond this torso distance while it is idle.
    approach: f32,
    /// Back off below this distance while idle (0 = never).
    retreat: f32,
    /// A telegraph is answered once its windup progress passes this.
    react_at: f32,
    /// Strike only from at most this distance.
    strike_max: f32,
}

fn tune(s: StyleId) -> Tune {
    match s {
        StyleId::Press => Tune { approach: 1.75, retreat: 1.1, react_at: 0.4, strike_max: 2.2 },
        StyleId::Counter => Tune { approach: 1.98, retreat: 1.8, react_at: 0.0, strike_max: 2.15 },
        StyleId::Duelist => Tune { approach: 2.02, retreat: 1.85, react_at: 0.15, strike_max: 2.2 },
    }
}

pub struct StyleBot {
    style: StyleId,
    /// Per-fight personality jitter (spacing and reaction), fixed by the seed.
    jitter: [f32; 3],
    last_x: f32,
    stuck: f32,
    /// Neutral attacks launched so far (selects the next move of a band's rotation).
    count: u32,
}

impl StyleBot {
    pub fn new(style: StyleId, seed: u64) -> StyleBot {
        let mut rng = Rng::new(seed ^ 0x57_1E_B07 ^ ((style.index() as u64) << 40));
        let jitter = [rng.range(-0.12, 0.12), rng.range(-0.1, 0.1), rng.range(-0.08, 0.08)];
        StyleBot { style, jitter, last_x: f32::NAN, stuck: 0.0, count: 0 }
    }

    pub fn style(&self) -> StyleId {
        self.style
    }

    /// The attack for the situation: a pure function of distance and the opponent's state.
    fn pick_attack(&self, dist: f32, theirs: &MoveInfo, opp_open: bool) -> (MoveId, MoveParams) {
        let guard_up = theirs.id == MoveId::Guard && theirs.phase != Phase::Idle;
        let st = self.style;
        if dist < CLOSE {
            return if guard_up || self.count % 3 == 2 { (MoveId::ShieldBash, p(0.8, 0.9, 0.5, 0.0)) } else { (MoveId::Kick, p(0.8, 0.9, 0.5, 0.0)) };
        }
        if guard_up && dist < 1.7 {
            return (MoveId::ShieldBash, p(0.8, 0.9, 0.5, 0.0));
        }
        if opp_open {
            let n = self.count as usize;
            let far = [MoveId::DiagUp, MoveId::Thrust, MoveId::DiagDown, MoveId::Overhead];
            let near = [MoveId::Thrust, MoveId::DiagUp, MoveId::LowCut, MoveId::DiagDown];
            let id = if dist > 1.7 { far[n % 4] } else { near[n % 4] };
            return (id, p(0.95, 1.0, 0.5, 0.0));
        }
        if guard_up {
            return (MoveId::LowCut, p(0.9, 0.95, 0.5, 0.0));
        }
        if theirs.id == MoveId::Backstep && theirs.phase != Phase::Idle {
            return (MoveId::Sweep, p(1.0, 1.0, 0.5, 0.0));
        }
        // Neutral attacks rotate through a few moves per distance band, so a fight shows the whole vocabulary.
        let n = self.count as usize;
        let pick = |list: &[(MoveId, MoveParams)]| list[n % list.len()];
        match st {
            StyleId::Press => {
                let q = p(0.95, 1.0, 0.45, 0.0);
                if dist > 1.95 {
                    pick(&[(MoveId::DiagUp, q), (MoveId::Thrust, q), (MoveId::DiagUp, q), (MoveId::Sweep, q)])
                } else if dist > 1.7 {
                    (MoveId::Overhead, p(0.95, 1.0, 0.45, 0.0))
                } else if dist > 1.5 {
                    (MoveId::DiagDown, p(0.95, 1.0, 0.45, 0.0))
                } else {
                    (MoveId::Thrust, p(0.9, 1.0, 0.45, 0.0))
                }
            }
            StyleId::Counter => {
                if dist > 1.9 {
                    (MoveId::DiagUp, p(0.9, 0.95, 0.45, 0.0))
                } else if dist > 1.6 {
                    (MoveId::Overhead, p(0.9, 0.95, 0.45, 0.0))
                } else {
                    (MoveId::DiagDown, p(0.9, 0.95, 0.45, 0.0))
                }
            }
            StyleId::Duelist => {
                if dist > 1.95 {
                    (MoveId::Thrust, p(1.0, 0.95, 0.5, 0.0))
                } else if dist > 1.7 {
                    (MoveId::DiagUp, p(0.95, 0.95, 0.5, 0.55))
                } else {
                    (MoveId::DiagDown, p(0.9, 0.95, 0.5, 0.55))
                }
            }
        }
    }

    /// Answer to a telegraphed attack, chosen by what is coming.
    fn defend(&self, theirs: &MoveInfo, dist: f32) -> Request {
        if dist > 2.7 {
            return None;
        }
        match theirs.id {
            MoveId::Thrust => {
                if self.style == StyleId::Press {
                    Some((MoveId::Parry, p(0.5, 0.9, 0.5, 0.0)))
                } else {
                    Some((MoveId::Backstep, p(0.8, 0.9, 0.5, 0.0)))
                }
            }
            MoveId::Overhead | MoveId::DiagDown => Some((MoveId::Parry, p(0.5, 0.9, 0.5, 0.0))),
            MoveId::DiagUp | MoveId::LowCut | MoveId::Sweep | MoveId::Kick | MoveId::ShieldBash | MoveId::Riposte => {
                match self.style {
                    StyleId::Press => None,
                    StyleId::Counter => Some((MoveId::Parry, p(0.5, 0.9, 0.5, 0.0))),
                    StyleId::Duelist => Some((MoveId::Guard, p(0.5, 0.8, 0.5, 0.0))),
                }
            }
            _ => None,
        }
    }

    /// One tick: the move request and the footwork drive.
    pub fn act(&mut self, me: &Robot, opp: &Robot, mine: &MoveInfo, theirs: &MoveInfo, _live_threat: bool, dt: f32) -> (Request, f32) {
        let mut t = tune(self.style);
        t.approach += self.jitter[0];
        t.retreat += self.jitter[0] * 0.8;
        t.react_at = (t.react_at + self.jitter[1]).max(0.0);
        t.strike_max += self.jitter[2];
        let dx = opp.x - me.x;
        let dist = dx.abs();
        let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
        let idle = mine.phase == Phase::Idle;
        let down = opp.state() == State::Fallen;
        let stag = opp.state() == State::Staggered;
        let opp_open = theirs.open || stag || down;
        let their_attack = is_attack(theirs.id) && matches!(theirs.phase, Phase::Windup | Phase::Active);
        let guard_up = theirs.id == MoveId::Guard && theirs.phase != Phase::Idle;
        let opp_idle = theirs.phase == Phase::Idle;

        // Footwork.
        let mut drive = 0.0;
        if down {
            if dist > 1.6 {
                drive = dir;
            }
        } else if their_attack && self.style != StyleId::Press && dist < 2.9 {
            drive = -dir;
        } else if their_attack && dist > t.strike_max {
            drive = 0.0;
        } else if opp_open && dist > 1.8 {
            drive = dir;
        } else if guard_up && dist > 1.5 {
            drive = dir;
        } else if dist > t.approach {
            drive = dir;
        } else if dist < t.retreat && idle && !opp_open {
            drive = -dir * 0.7;
        }
        // After a swing the duelist and counter-fighter slip back out of reach while recovering.
        if mine.phase == Phase::Recovery && is_attack(mine.id) && matches!(self.style, StyleId::Duelist | StyleId::Counter) && !opp_open {
            drive = -dir;
        }
        if drive != 0.0 && (me.x - self.last_x).abs() < 1e-5 {
            self.stuck += dt
        } else {
            self.stuck = 0.0
        }
        self.last_x = me.x;
        let stuck = self.stuck > 0.3;

        // Riposte right after a successful parry (all styles).
        if mine.riposte_remaining > 0.0 && idle && dist <= SWING_MAX + 0.3 {
            return (Some((MoveId::Riposte, p(0.9, 1.0, 0.5, 0.0))), drive);
        }
        // Feint: the swing would meet a raised parry, so cancel it and strike once the parry is spent.
        if self.style == StyleId::Duelist && mine.phase == Phase::Windup && mine.progress >= 0.35 && mine.progress <= 0.55 && is_attack(mine.id) && theirs.id == MoveId::Parry && theirs.phase != Phase::Idle {
            return (Some((MoveId::Feint, p(0.5, 0.5, 0.5, 0.0))), drive);
        }
        // Answer a telegraph once it is committed.
        let telegraph = their_attack && theirs.phase == Phase::Windup && theirs.progress >= t.react_at || their_attack && theirs.phase == Phase::Active;
        if telegraph && idle && !opp_open {
            if let Some(d) = self.defend(theirs, dist) {
                return (Some(d), drive);
            }
        }
        // Counter: a defended swing leaves the attacker in recovery; the counter-fighter answers at once.
        let in_range = dist <= if stuck { STUCK_RANGE } else if down { 1.85 } else { t.strike_max };
        let chain = mine.phase == Phase::Recovery && mine.progress >= 0.75 && self.style == StyleId::Press && is_attack(mine.id);
        if (idle || chain) && in_range {
            let parry_up = theirs.id == MoveId::Parry && matches!(theirs.phase, Phase::Windup | Phase::Active);
            let parry_spent = theirs.id == MoveId::Parry && theirs.phase == Phase::Recovery;
            let punishable = opp_open || parry_spent || theirs.phase == Phase::Recovery && is_attack(theirs.id);
            let allowed = match self.style {
                StyleId::Press => (!their_attack || opp_open) && !parry_up,
                StyleId::Counter => (opp_idle || punishable) && !their_attack,
                StyleId::Duelist => (opp_idle || punishable || guard_up) && !their_attack,
            };
            if down {
                let (id, pr) = if dist < 1.5 { (MoveId::Thrust, p(0.9, 1.0, 0.5, 0.0)) } else { (MoveId::Overhead, p(0.9, 1.0, 0.5, 0.0)) };
                return (Some((id, pr)), drive);
            }
            if allowed && (dist >= 1.0 || stuck) {
                let (id, pr) = self.pick_attack(dist, theirs, opp_open || punishable);
                return (Some((id, pr)), drive);
            }
        }
        (None, drive)
    }
}
