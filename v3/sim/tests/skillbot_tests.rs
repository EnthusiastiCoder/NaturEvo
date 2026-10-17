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

fn starts(a: Brain, b: Brain, seed: u64) -> (Vec<MoveId>, Vec<MoveId>) {
    let mut m = Match::new(seed, a, b);
    m.run_to_end();
    (m.move_starts(0), m.move_starts(1))
}

#[test]
fn skillbot_reports_its_tier() {
    for t in 1..=3u8 { assert_eq!(SkillBot::new(5, t).tier(), t); }
}

#[test]
fn same_seed_same_tier_same_move_sequence() {
    for t in 1..=3u8 {
        let a = starts(Brain::Skill(t), Brain::Scripted, 11);
        let b = starts(Brain::Skill(t), Brain::Scripted, 11);
        assert_eq!(a, b, "tier {}", t);
        assert!(!a.0.is_empty(), "tier {} never attacked", t);
    }
}
