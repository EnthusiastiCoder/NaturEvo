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

#[test]
fn different_seeds_give_different_sequences() {
    let a = starts(Brain::Skill(3), Brain::Scripted, 1).0;
    assert!((2..12u64).any(|s| starts(Brain::Skill(3), Brain::Scripted, s).0 != a));
}

#[test]
fn tier_1_attacks_with_more_than_one_kind_of_move() {
    let mut seen = std::collections::BTreeSet::new();
    for s in 0..15u64 { for m in starts(Brain::Skill(1), Brain::Dummy, s).0 { seen.insert(m as usize); } }
    assert!(seen.len() >= 4, "brawler used {:?}", seen);
    assert!(!seen.contains(&(MoveId::Feint as usize)), "tier 1 does not feint");
}

#[test]
fn tier_3_uses_at_least_10_distinct_moves_over_30_seeds() {
    let mut seen = std::collections::BTreeSet::new();
    for s in 0..30u64 {
        let opp = if s % 2 == 0 { Brain::Skill(2) } else { Brain::Scripted };
        for m in starts(Brain::Skill(3), opp, s).0 { seen.insert(m as usize); }
    }
    assert!(seen.len() >= 10, "duellist used only {} of {}: {:?}", seen.len(), N_MOVES, seen);
    for must in [MoveId::Feint, MoveId::Parry, MoveId::Guard] {
        assert!(seen.contains(&(must as usize)), "{:?} never used", must);
    }
}

#[test]
fn tier_1_beats_dummy_in_20_of_20_seeds() {
    for s in 0..20u64 {
        let r = Match::new(s, Brain::Skill(1), Brain::Dummy).run_to_end();
        assert_eq!(r.outcome, Outcome::WinA, "seed {}", s);
    }
}

#[test]
fn tier_2_and_3_also_beat_dummy_decisively() {
    for t in [2u8, 3] {
        let wins = (0..10u64).filter(|s| Match::new(*s, Brain::Skill(t), Brain::Dummy).run_to_end().outcome == Outcome::WinA).count();
        assert!(wins >= 9, "tier {} won {}/10", t, wins);
    }
}
