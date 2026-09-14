use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;
use arena_sim::matchup::{play, Outcome};
use arena_sim::world::Arena;

#[test]
fn late_champion_fights_are_decisive_and_use_all_actions() {
    let mut t = Trainer::new(TrainConfig { seed: 1, generations: 40, pop: 24, k_opponents: 4, hof_sample: 2 });
    for _ in 0..40 { t.run_generation(); }
    let cfg = ArenaConfig::default();
    let hh = t.hall_of_fame(Role::Hero);
    let mh = t.hall_of_fame(Role::Monster);
    let (mut draws, mut n) = (0, 0);
    let mut share = [0.0f64; 3];
    let mut frames = 0.0f64;
    for k in 0..5 {
        let (h, m) = (&hh[hh.len() - 1 - k], &mh[mh.len() - 1 - k]);
        for s in 0..8u64 {
            let r = play(Arena::generate(s, &cfg), h.clone(), m.clone(), s, &cfg);
            n += 1;
            if r.outcome == Outcome::Draw { draws += 1; }
            for f in &r.log.frames {
                for ri in 0..2 {
                    let a = &f.action[ri];
                    share[0] += a.attack as u8 as f64; share[1] += a.block as u8 as f64; share[2] += a.dodge as u8 as f64;
                }
                frames += 2.0;
            }
        }
    }
    let sh: Vec<f64> = share.iter().map(|x| x / frames).collect();
    eprintln!("METRIC draws {draws}/{n} shares atk {:.3} blk {:.3} dge {:.3}", sh[0], sh[1], sh[2]);
    assert!((draws as f64) / (n as f64) < 0.2, "draw rate {draws}/{n}");
    for (i, s) in sh.iter().enumerate() { assert!(*s >= 0.02, "action {i} share {s}"); }
}
