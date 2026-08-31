// Assumed Map API: to_json(), camp(), boss() -> (i32,i32), is_blocked((i32,i32)), astar(a,b)->Option<Vec<(i32,i32)>>,
// zone_tier(tile)->u32, biome_count(), blocked_count(), first_blocked()->Option<(i32,i32)>.
use arena_sim::jungle::Map;

#[test]
fn generation_deterministic_by_seed() {
    assert_eq!(Map::generate(5, 40).to_json(), Map::generate(5, 40).to_json());
    assert_ne!(Map::generate(5, 40).to_json(), Map::generate(6, 40).to_json());
}

#[test]
fn camp_and_boss_reachable_over_many_seeds() {
    for seed in 0..30u64 {
        let m = Map::generate(seed, 40);
        assert!(!m.is_blocked(m.camp()), "camp blocked seed {seed}");
        assert!(!m.is_blocked(m.boss()), "boss blocked seed {seed}");
        let p = m.astar(m.camp(), m.boss()).unwrap_or_else(|| panic!("no route seed {seed}"));
        assert_eq!(p.first().copied(), Some(m.camp()));
        assert_eq!(p.last().copied(), Some(m.boss()));
        for w in p.windows(2) {
            let (dx, dy) = ((w[0].0 - w[1].0).abs(), (w[0].1 - w[1].1).abs());
            assert!(dx <= 1 && dy <= 1 && dx + dy > 0);
            assert!(!m.is_blocked(w[1]));
        }
    }
}

#[test]
fn boss_zone_tier_above_camp_zone() {
    let m = Map::generate(3, 40);
    assert!(m.zone_tier(m.boss()) > m.zone_tier(m.camp()));
}

#[test]
fn has_biomes_and_obstacles() {
    let m = Map::generate(3, 40);
    assert!(m.biome_count() >= 2);
    assert!(m.blocked_count() > 0);
    assert!(m.blocked_count() < 40 * 40 / 2);
}
