// Assumed test hooks beyond the contract (listed in the report):
// Match::step_with(Action, Action), place_for_duel(dist) (face each other, open ground),
// face_away(Role), set_stamina(Role, f32).
use arena_sim::config::ArenaConfig;
use arena_sim::fighter::{Action, Role};
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Match, Outcome};
use arena_sim::rng::Rng;
use arena_sim::world::Arena;

fn idle() -> Action { Action { fwd: 0.0, strafe: 0.0, turn: 0.0, attack: false, block: false, dodge: false } }
fn atk() -> Action { Action { attack: true, ..idle() } }
fn blk() -> Action { Action { block: true, ..idle() } }
fn dge() -> Action { Action { dodge: true, ..idle() } }
