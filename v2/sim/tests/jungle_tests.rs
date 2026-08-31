// Assumed Map API: to_json(), camp(), boss() -> (i32,i32), is_blocked((i32,i32)), astar(a,b)->Option<Vec<(i32,i32)>>,
// zone_tier(tile)->u32, biome_count(), blocked_count(), first_blocked()->Option<(i32,i32)>.
use arena_sim::jungle::Map;
