// r6: the style training opponent mix must contain single-move spammers so nets learn to counter spam.
use mech_sim::evolve::{style_mix, SKind};
use mech_sim::stylebot::StyleId;

#[test]
fn style_mix_after_warmup_contains_spammers_and_sums_to_one() {
    for st in StyleId::ALL {
        let mix = style_mix(500, 1000, st, true);
        let spam: f32 = mix.iter().filter(|(k, _)| matches!(k, SKind::Spam(_))).map(|(_, w)| *w).sum();
        assert!(spam >= 0.08, "spam weight {}", spam);
        let kinds = mix.iter().filter(|(k, _)| matches!(k, SKind::Spam(_))).count();
        assert!(kinds >= 3, "{} spam kinds", kinds);
        let total: f32 = mix.iter().map(|(_, w)| w).sum();
        assert!((total - 1.0).abs() < 1e-3, "weights sum {}", total);
    }
}

#[test]
fn every_style_bonus_pays_for_landing_more_distinct_moves() {
    use mech_sim::duel::{Brain, Match};
    use mech_sim::evolve::style_bonus;
    let mut m = Match::new(3, Brain::Skill(2), Brain::Skill(1));
    let mut r = m.run_to_end();
    // Make sure the fight is scored (not capped, not drawn).
    r.capped = false;
    r.outcome = mech_sim::duel::Outcome::WinA;
    for st in StyleId::ALL {
        r.landed_moves = [1, 1];
        let one = style_bonus(st, &r, 0);
        r.landed_moves = [5, 1];
        let five = style_bonus(st, &r, 0);
        assert!(five > one + 0.5, "{} bonus {} -> {}", st.name(), one, five);
    }
}
