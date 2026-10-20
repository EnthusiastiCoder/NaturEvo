//! Deterministic move executor: (move, params) -> smooth keyframe joint targets.
//! State machine Idle -> Windup -> Active -> Recovery -> Idle. Active is never cancelled; Windup is cancelled only
//! by a Feint (before 60% of the windup) or by a threatened Backstep; the last 25% of Recovery accepts a chain.
//! The executor outputs the same `Action` as scripted.rs. Energy is only tracked here (`energy_spent`); the duel
//! applies it to the robot.
use crate::config::*;
use crate::moves::move_possible_id;
use crate::moveset::*;
use crate::robot::Robot;
use crate::scripted::Action;

#[derive(Clone, Debug)]
pub struct Executor {
    id: MoveId,
    params: MoveParams,
    phase: Phase,
    elapsed: f32,
    dur: [f32; 3],
    contact: bool,
    whiffed: bool,
    committed: bool,
    cancelled: bool,
    cancel_from: [f32; 12],
    start_pose: [f32; 12],
    spent: f32,
    hist1: [f32; 12],
    hist2: [f32; 12],
    last_pose: [f32; 12],
    threat: bool,
    riposte_open: bool,
    facing: f32,
}

const DEFAULT_PARAMS: MoveParams = MoveParams { reach: 0.5, speed: 0.5, arc: 0.5, delay: 0.0 };

impl Default for Executor {
    fn default() -> Self {
        Executor::new()
    }
}

fn is_attack(id: MoveId) -> bool {
    !matches!(id, MoveId::Idle | MoveId::Feint)
}

fn move_energy(id: MoveId, p: &MoveParams) -> f32 {
    spec(id).energy * (0.6 + 0.8 * p.speed.clamp(0.0, 1.0)) + DRAG_COST * p.delay.clamp(0.0, 1.0)
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            id: MoveId::Idle,
            params: DEFAULT_PARAMS,
            phase: Phase::Idle,
            elapsed: 0.0,
            dur: [0.0; 3],
            contact: false,
            whiffed: false,
            committed: false,
            cancelled: false,
            cancel_from: REST,
            start_pose: REST,
            spent: 0.0,
            hist1: REST,
            hist2: REST,
            last_pose: REST,
            threat: false,
            riposte_open: false,
            facing: 1.0,
        }
    }

    pub fn move_id(&self) -> MoveId {
        self.id
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn params(&self) -> MoveParams {
        self.params
    }
    /// Progress 0..=1 inside the current phase.
    pub fn progress(&self) -> f32 {
        if self.phase == Phase::Idle {
            return 0.0;
        }
        (self.elapsed / self.cur_dur().max(1e-6)).clamp(0.0, 1.0)
    }
    /// True from Active start until the move ends.
    pub fn committed(&self) -> bool {
        self.committed
    }
    /// Active ended without contact; stays set through Recovery.
    pub fn whiffed(&self) -> bool {
        self.whiffed
    }
    pub fn contact_made(&self) -> bool {
        self.contact
    }
    /// The caller reports the move touched the opponent during Active.
    pub fn notify_contact(&mut self) {
        if self.phase == Phase::Active {
            self.contact = true;
        }
    }
    /// Cumulative energy cost charged by the executor (move, feint, drag, chain, backstep cancel, jerk).
    pub fn energy_spent(&self) -> f32 {
        self.spent
    }
    /// A live threat exists (the caller decides): lets a Backstep request cancel a windup.
    pub fn set_threat(&mut self, threat: bool) {
        self.threat = threat;
    }
    /// A riposte window is open: a Riposte started now winds up in `RIPOSTE_WINDUP_OPEN`.
    pub fn set_riposte_open(&mut self, open: bool) {
        self.riposte_open = open;
    }
    /// Cancelled windup relaxing back to the ready pose (a feint in progress).
    pub fn cancelling(&self) -> bool {
        self.cancelled
    }

    fn cur_dur(&self) -> f32 {
        match self.phase {
            Phase::Idle => 0.0,
            Phase::Windup => self.dur[0],
            Phase::Active => self.dur[1],
            Phase::Recovery => {
                if self.cancelled {
                    CANCEL_RECOVER
                } else {
                    self.dur[2]
                }
            }
        }
    }

    fn reset_move(&mut self) {
        self.id = MoveId::Idle;
        self.phase = Phase::Idle;
        self.elapsed = 0.0;
        self.committed = false;
        self.cancelled = false;
        self.whiffed = false;
        self.contact = false;
    }

    fn advance(&mut self, dt: f32) {
        if self.phase == Phase::Idle {
            return;
        }
        self.elapsed += dt;
        loop {
            let d = self.cur_dur();
            if self.elapsed + 1e-5 < d {
                break;
            }
            self.elapsed = (self.elapsed - d).max(0.0);
            match self.phase {
                Phase::Windup => {
                    self.phase = Phase::Active;
                    self.committed = true;
                    self.contact = false;
                }
                Phase::Active => {
                    self.phase = Phase::Recovery;
                    self.whiffed = spec(self.id).can_whiff && !self.contact;
                }
                Phase::Recovery => {
                    self.reset_move();
                    break;
                }
                Phase::Idle => break,
            }
        }
    }

    fn begin(&mut self, id: MoveId, p: MoveParams, extra_cost: f32) {
        let mut d = phase_durations(id, &p);
        if id == MoveId::Riposte && self.riposte_open {
            d[0] = RIPOSTE_WINDUP_OPEN * speed_mult(p.speed) + DELAY_MAX * p.delay.clamp(0.0, 1.0);
        }
        self.id = id;
        self.params = p;
        self.dur = d;
        self.phase = Phase::Windup;
        self.elapsed = 0.0;
        self.committed = false;
        self.cancelled = false;
        self.whiffed = false;
        self.contact = false;
        self.start_pose = self.last_pose;
        self.spent += move_energy(id, &p) + extra_cost;
    }

    fn handle_request(&mut self, me: &Robot, id: MoveId, p: MoveParams) {
        let in_windup = self.phase == Phase::Windup && !self.cancelled;
        if id == MoveId::Idle || self.phase == Phase::Active {
            return;
        }
        if id == MoveId::Feint {
            if in_windup && self.progress() < FEINT_CANCEL_FRAC && me.energy >= FEINT_COST {
                self.cancel();
                self.spent += FEINT_COST;
            }
            return;
        }
        if id == MoveId::Backstep && in_windup && self.threat && self.id != MoveId::Backstep {
            let cost = move_energy(id, &p) + BACKSTEP_CANCEL_COST;
            if me.energy >= cost && move_possible_id(me, id) {
                self.begin(id, p, BACKSTEP_CANCEL_COST);
            }
            return;
        }
        let idle = self.phase == Phase::Idle;
        let chain = self.phase == Phase::Recovery && self.progress() >= 1.0 - CHAIN_FRAC;
        if !(idle || chain) || !is_attack(id) {
            return;
        }
        let extra = if chain { CHAIN_COST } else { 0.0 };
        if me.energy < move_energy(id, &p) + extra || !move_possible_id(me, id) {
            return;
        }
        self.begin(id, p, extra);
    }
