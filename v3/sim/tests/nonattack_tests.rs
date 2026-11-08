// A fighter that only requests defensive moves (Parry, Guard, Backstep) must not hurt anyone: only attack moves
// deal sword damage (found when trained nets won by spamming Parry sweeps through the opponent's body).
use mech_sim::duel::{Brain, Match};
use mech_sim::moveset::MoveId;

#[test]
fn spamming_defensive_moves_deals_no_damage() {
    for id in [MoveId::Parry, MoveId::Guard, MoveId::Backstep] {
        let mut total_hits = 0;
        for seed in 1..=6u64 {
            let mut m = Match::new(seed, Brain::Spam(id), Brain::Dummy);
            let r = m.run_to_end();
            total_hits += r.hits[0];
        }
        assert_eq!(total_hits, 0, "{:?} dealt hits", id);
    }
}
