// r7: style identity in training fitness. Move share entropy (not distinct count), footwork, spacing spread,
// counter punish, and graded capped-fight loss. Nothing here pays for guarding, blocking or parrying.
use mech_sim::duel::{Brain, Match, MatchResult, Outcome};
use mech_sim::evolve::style_bonus;
use mech_sim::fitness::move_entropy;
use mech_sim::moveset::MoveId;
use mech_sim::stylebot::StyleId;

fn base() -> MatchResult {
    let mut m = Match::new(3, Brain::Skill(2), Brain::Skill(1));
    let mut r = m.run_to_end();
    r.capped = false;
    r.outcome = Outcome::WinA;
    r.landed_moves = [3, 3];
    r.swings = [12, 12];
    r.move_counts = [[0; 14]; 2];
    r.hit_counts = [[0; 14]; 2];
    for k in [1usize, 2, 3, 4, 5, 6] {
        r.move_counts[0][k] = 2;
        r.hit_counts[0][k] = 1;
    }
    r.still_share = [0.1, 0.1];
    r.fwd_share = [0.3, 0.3];
    r.engaged_std = 0.6;
    r.counter_hits = [2, 0];
    r.backstep_hits = [1, 0];
    r
}

#[test]
fn move_entropy_is_zero_for_one_move_and_high_for_a_spread() {
    let mut one = [0u16; 14];
    one[MoveId::DiagUp as usize] = 20;
    assert!(move_entropy(&one) < 0.01);
    let mut many = [0u16; 14];
    for k in [1usize, 2, 3, 4, 5, 6] {
        many[k] = 4;
    }
    assert!(move_entropy(&many) > 0.85, "{}", move_entropy(&many));
}

#[test]
fn match_result_counts_moves_started_per_fighter() {
    let mut m = Match::new(5, Brain::Skill(2), Brain::Skill(1));
    let r = m.run_to_end();
    for i in 0..2 {
        assert_eq!(r.move_counts[i].iter().map(|&c| c as u32).sum::<u32>(), r.swings[i]);
    }
    assert!(r.swings[0] + r.swings[1] > 0);
    let mut m = Match::new(5, Brain::Skill(2), Brain::Dummy);
    let r = m.run_to_end();
    assert_eq!(r.move_counts[1].iter().map(|&c| c as u32).sum::<u32>(), 0);
    assert!((0.0..=1.0).contains(&r.still_share[1]) && r.still_share[1] > 0.9, "dummy still {}", r.still_share[1]);
}

#[test]
fn every_style_prefers_spread_over_one_move_spam() {
    for st in StyleId::ALL {
        let spread = base();
        let mut spam = base();
        spam.move_counts[0] = [0; 14];
        spam.move_counts[0][MoveId::DiagUp as usize] = 12;
        spam.hit_counts[0] = [0; 14];
        spam.hit_counts[0][MoveId::DiagUp as usize] = 6;
        assert!(style_bonus(st, &spread, 0) > style_bonus(st, &spam, 0) + 0.5, "{}", st.name());
    }
}

#[test]
fn every_style_penalises_standing_still() {
    for st in StyleId::ALL {
        let mut still = base();
        still.still_share = [0.6, 0.6];
        assert!(style_bonus(st, &base(), 0) > style_bonus(st, &still, 0) + 0.5, "{}", st.name());
    }
}

#[test]
fn counter_pays_for_converting_a_parry_or_backstep_into_a_hit_not_for_the_parry() {
    let mut none = base();
    none.counter_hits = [0, 0];
    none.backstep_hits = [0, 0];
    let more = base();
    assert!(style_bonus(StyleId::Counter, &more, 0) > style_bonus(StyleId::Counter, &none, 0) + 0.5);
    // Parries without a follow-up hit earn nothing extra.
    let mut parried = none.clone();
    parried.defence[0].parry += 10;
    assert_eq!(style_bonus(StyleId::Counter, &parried, 0), style_bonus(StyleId::Counter, &none, 0));
}

#[test]
fn press_pays_forward_steps_and_kicks_duelist_pays_spacing_spread() {
    let mut fwd = base();
    fwd.fwd_share = [0.5, 0.5];
    let mut stay = base();
    stay.fwd_share = [0.05, 0.05];
    assert!(style_bonus(StyleId::Press, &fwd, 0) > style_bonus(StyleId::Press, &stay, 0) + 0.3);
    let mut kick = base();
    kick.move_counts[0][MoveId::Kick as usize] = 3;
    kick.move_counts[0][MoveId::LowCut as usize] = 3;
    let mut nokick = base();
    nokick.move_counts[0][MoveId::Kick as usize] = 0;
    nokick.move_counts[0][MoveId::LowCut as usize] = 0;
    assert!(style_bonus(StyleId::Press, &kick, 0) > style_bonus(StyleId::Press, &nokick, 0));
    let mut wide = base();
    wide.engaged_std = 0.9;
    let mut flat = base();
    flat.engaged_std = 0.1;
    assert!(style_bonus(StyleId::Duelist, &wide, 0) > style_bonus(StyleId::Duelist, &flat, 0) + 0.3);
}

#[test]
fn capped_fights_cost_every_style_and_counter_must_have_converted_a_hit() {
    for st in StyleId::ALL {
        let mut cap = base();
        cap.capped = true;
        cap.outcome = Outcome::Draw;
        cap.hits = [0, 0];
        cap.counter_hits = [0, 0];
        cap.backstep_hits = [0, 0];
        let none = style_bonus(st, &cap, 0);
        assert!(none < -1.0, "{} capped bonus {}", st.name(), none);
        let mut conv = cap.clone();
        conv.hits = [3, 0];
        conv.counter_hits = [3, 0];
        assert!(style_bonus(st, &conv, 0) > none + 0.5, "{}", st.name());
    }
}
