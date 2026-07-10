use jungle_sim::config::{knobs, Config};
use jungle_sim::report::batch;
use jungle_sim::sim::{simulate, simulate_with};

#[test]
fn default_config_values() {
    let c = Config::default();
    assert_eq!(c.monster_hp_mult, 1.0);
    assert_eq!(c.monster_atk_mult, 1.0);
    assert_eq!(c.boss_hp_mult, 1.0);
    assert_eq!(c.boss_atk_mult, 1.0);
    assert_eq!(c.hero_hp_base, 50);
    assert_eq!(c.hero_hp_per_level, 14);
    assert_eq!(c.hero_atk_per_level, 3.0);
    assert_eq!(c.hero_def_per_level, 1.5);
    assert_eq!(c.xp_mult, 1.0);
    assert_eq!(c.gold_mult, 1.0);
    assert_eq!(c.potion_cost, 25);
    assert_eq!(c.boss_level, 32);
    assert_eq!(c.boss_attempts, 8);
    assert_eq!(c.event_cap, 65536);
    assert_eq!(c.postgame_level_cap, 80);
    assert_eq!(c.evolve_kills, 300);
}
