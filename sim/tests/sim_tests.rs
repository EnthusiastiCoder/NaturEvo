use jungle_sim::data::SPECIES;
use jungle_sim::sim::{monster_stat, simulate, xp_to_next};

const KINDS: [&str; 15] = [
    "travel", "encounter", "attack", "skill", "hit_taken", "potion", "level_up",
    "victory_fight", "death", "respawn", "shop", "rest", "boss", "win", "evolve",
];

#[test]
fn same_seed_identical() {
    let a = simulate(7);
    let b = simulate(7);
    assert_eq!(a.events.len(), b.events.len());
    for (x, y) in a.events.iter().zip(b.events.iter()) {
        assert_eq!(x.text, y.text);
        assert_eq!(x.kind, y.kind);
    }
    let (x, y) = (a.events.last().unwrap(), b.events.last().unwrap());
    assert_eq!(x.hero.level, y.hero.level);
    assert_eq!(x.hero.hp, y.hero.hp);
    assert_eq!(x.hero.gold, y.hero.gold);
    assert_eq!(a.won, b.won);
    assert_eq!(a.deaths, b.deaths);
    assert_eq!(a.final_level, b.final_level);
}
