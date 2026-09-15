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

#[test]
fn late_checkpoints_beat_earlier_ones_for_both_sides() {
    let gens = 150usize;
    let mut t = Trainer::new(TrainConfig { seed: 1, generations: gens as u32, pop: 48, k_opponents: 4, hof_sample: 8 });
    for _ in 0..gens { t.run_generation(); }
    for role in [Role::Hero, Role::Monster] {
        let h = t.hall_of_fame(role);
        let panel: Vec<Genome> = t.hall_of_fame(role.other()).iter().step_by(3).cloned().collect();
        assert!(h.len() >= gens, "hall of fame must keep every champion");
        let late = &h[gens - 1];
        for early in [10usize, 50] {
            let (l, e) = late_share(role, late, &h[early], &panel, 8);
            eprintln!("PROGRESS {:?} late vs gen{early}: late {l} early {e}", role);
            assert!(l + e >= 8, "{:?} too few discordant fights {l}/{e}", role);
            assert!(l as f64 >= 0.75 * (l + e) as f64, "{:?} gen{} lost to gen{early}: {l} vs {e}", role, gens - 1);
        }
    }
}

#[test]
#[ignore]
fn probe_saved_run_progress() {
    let d = std::env::var("RUN_DIR").unwrap();
    let ld = |r: &str, g: u32| Genome::from_json(&std::fs::read_to_string(format!("{d}/champion_{r}_gen_{g}.json")).unwrap()).unwrap();
    for (role, name, other) in [(Role::Hero, "hero", "monster"), (Role::Monster, "monster", "hero")] {
        let panel: Vec<Genome> = (0..20).map(|k| ld(other, k * 10 + 9)).collect();
        for early in [9u32, 49] {
            let (l, e) = late_share(role, &ld(name, 199), &ld(name, early), &panel, 8);
            eprintln!("SAVED {name} 199 vs {early}: late {l} early {e} share {:.2}", l as f64 / (l + e).max(1) as f64);
        }
    }
}
