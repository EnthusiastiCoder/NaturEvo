// WS-C SkillBot tests (plan 2026-10-23_v3-sword-vocabulary.md, Test cases WS-C).
//
// INVENTED API:
//   mech_sim::skillbot (new file v3/sim/src/skillbot.rs):
//     pub struct SkillBot;  impl SkillBot { pub fn new(seed: u64, tier: u8) -> SkillBot /* tier 1..=3 */; pub fn tier(&self) -> u8; }
//   mech_sim::duel::Brain gains `Skill(u8)` (tier 1 Brawler, 2 Reactive, 3 Duellist), driven through the same Executor as the net.
//   mech_sim::duel::Brain also gains two TEST-SUPPORT brains driven through the Executor:
//     `Spam(MoveId)`: walks toward the opponent (drive +1 until within ~2.2 m) and requests this move (mid params) whenever the
//                     executor can start one; never defends.
//     `Cycle(Vec<MoveId>)`: same walking, requests the listed moves in round-robin order.
//   mech_sim::duel::Match gains `pub fn move_starts(&self, who: usize) -> Vec<MoveId>` (every move the executor of `who`
//     accepted, in order, Feint included).
//   Log events (log.rs): see vocab_duel_tests.rs; `{"kind":"punish","who":W,"victim":V,...}`.
use mech_sim::duel::{Brain, Match, Outcome};
use mech_sim::log::fight_json;
use mech_sim::moveset::{MoveId, N_MOVES};
use mech_sim::skillbot::SkillBot;
