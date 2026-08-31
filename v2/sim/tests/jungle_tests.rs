// Assumed Map API: to_json(), camp(), boss() -> (i32,i32), is_blocked((i32,i32)), astar(a,b)->Option<Vec<(i32,i32)>>,
// zone_tier(tile)->u32, biome_count(), blocked_count(), first_blocked()->Option<(i32,i32)>.
use arena_sim::jungle::Map;

#[test]
fn generation_deterministic_by_seed() {
    assert_eq!(Map::generate(5, 40).to_json(), Map::generate(5, 40).to_json());
    assert_ne!(Map::generate(5, 40).to_json(), Map::generate(6, 40).to_json());
}
