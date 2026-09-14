use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;
use arena_sim::genome::Genome;
use arena_sim::matchup::{play, Outcome};
use arena_sim::world::Arena;

/// Share of discordant fights (one of the two same-side fighters wins, the other does not)
/// where `late` is the one that wins. Both face the same opposing panel on the same arenas.
fn late_share(role: Role, late: &Genome, early: &Genome, panel: &[Genome], seeds: u64) -> (u32, u32) {
    let cfg = ArenaConfig::default();
    let wins = |g: &Genome, o: &Genome, s: u64| {
        let (h, m) = if role == Role::Hero { (g.clone(), o.clone()) } else { (o.clone(), g.clone()) };
        let out = play(Arena::generate(s, &cfg), h, m, s, &cfg).outcome;
        matches!((out, role), (Outcome::HeroWin, Role::Hero) | (Outcome::MonsterWin, Role::Monster))
    };
    let (mut l, mut e) = (0, 0);
    for o in panel {
        for s in 0..seeds {
            match (wins(late, o, s), wins(early, o, s)) {
                (true, false) => l += 1,
                (false, true) => e += 1,
                _ => {}
            }
        }
    }
    (l, e)
}
