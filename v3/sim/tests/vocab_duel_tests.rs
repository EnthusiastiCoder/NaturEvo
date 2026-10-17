// WS-C duel integration tests for the vocabulary (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-C).
//
// INVENTED API (see skillbot_tests.rs for Brain::Skill, Brain::Spam, Brain::Cycle and Match::move_starts):
//   Log contract (additive, version stays 3), produced by log.rs when Match::enable_log() is on:
//     every fighter object in every frame gains `"mv":{"id":"Overhead","phase":0..3,"p":0..1,"open":bool,"gb":bool,"rip":bool}`
//       (phase: 0 Idle, 1 Windup, 2 Active, 3 Recovery);
//     events: {"t":..,"kind":"move","who":W,"name":"..","reach":..,"speed":..,"arc":..,"delay":..}
//             {"kind":"whiff","who","name"}  {"kind":"punish","who","victim","name","mult"}  {"kind":"guard_break","who","victim"}
//             {"kind":"feint","who"}  {"kind":"counter","who","name"}  {"kind":"riposte_open","who"}
//   Fighters controlled by Brain::Net execute through the Executor (net outputs decode to a move plus params).
use mech_sim::config::HARD_CAP_TICKS;
use mech_sim::duel::{Brain, Match};
use mech_sim::genome::Genome;
use mech_sim::log::fight_json;
use mech_sim::moveset::MoveId;
use mech_sim::rng::Rng;
use mech_sim::robot::{Part, Role};
