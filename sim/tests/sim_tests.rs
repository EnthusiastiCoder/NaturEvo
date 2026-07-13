use jungle_sim::data::SPECIES;
use jungle_sim::sim::{monster_stat, simulate, xp_to_next};

const KINDS: [&str; 15] = [
    "travel", "encounter", "attack", "skill", "hit_taken", "potion", "level_up",
    "victory_fight", "death", "respawn", "shop", "rest", "boss", "win", "evolve",
];
