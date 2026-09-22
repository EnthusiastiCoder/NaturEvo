// Multi-seed method: seeds 1,2,3 at smoke scale, final champions vs a held-out panel from the other seeds,
// 8 arena seeds, metrics via metrics::analyze. Means over seeds are compared to measured values plus a margin.
use arena_sim::config::ArenaConfig;
use arena_sim::evolve::{TrainConfig, Trainer};
use arena_sim::fighter::Role;
use arena_sim::matchup::{play_with_setup, Setup};
use arena_sim::attacks::{Rank, Species};
use arena_sim::metrics::analyze;
use arena_sim::world::Arena;

const GENS: u32 = 40;
// Placeholders: the WS4 implementer replaces these with the real measured means (comment the numbers).
// Assertions allow measured + 0.10 (draw rate) and measured + 0.05 (unseen block share).
const MEASURED_DRAW_RATE: f64 = 0.008; // measured mean draw rate over 3 seeds: 0.008 (0,1,0 of 40)
const MEASURED_UNSEEN_BLOCK: f64 = 0.0005; // measured mean unseen block share over 3 seeds: 0.0005

#[test]
fn late_champions_are_decisive_and_do_not_block_unseen_mean_over_three_seeds() {
    let trainers: Vec<Trainer> = (1..=3u64)
        .map(|seed| {
            let mut t = Trainer::new(TrainConfig { seed, generations: GENS, pop: 24, k_opponents: 4, hof_sample: 2 });
            for _ in 0..GENS { t.run_generation(); }
            t
        })
        .collect();
    let cfg = ArenaConfig::default();
    let (mut draw_mean, mut unseen_mean, mut action_mean) = (0.0f64, 0.0f64, [0.0f64; 3]);
    for (i, t) in trainers.iter().enumerate() {
        let other = &trainers[(i + 1) % 3];
        let hh = t.hall_of_fame(Role::Hero);
        let mh = other.hall_of_fame(Role::Monster);
        let (mut draws, mut n, mut unseen) = (0u32, 0u32, 0.0f64);
        let mut act = [0.0f64; 3];
        for k in 0..5 {
            let (h, m) = (&hh[hh.len() - 1 - k], &mh[mh.len() - 1 - k]);
            for s in 0..8u64 {
                let setup = Setup { hp: [cfg.max_hp, cfg.max_hp], species: [Species::Hero, Species::Minion], rank: [Rank::Grass, Rank::Grass] };
                let r = play_with_setup(Arena::generate(s, &cfg), h.clone(), m.clone(), s, &cfg, setup);
                let me = analyze(&r.log, Role::Hero, r.outcome);
                n += 1;
                if me.draw { draws += 1; }
                unseen += me.block_unseen_share() as f64;
                for a in 0..3 { act[a] += me.cells.iter().flatten().map(|d| d[a]).sum::<f32>() as f64; }
            }
        }
        let nf = n as f64;
        eprintln!("DECISIVE seed {} draws {draws}/{n} unseen_block {:.4} actions {:?}", i + 1, unseen / nf, act.map(|x| x / nf));
        draw_mean += draws as f64 / nf / 3.0;
        unseen_mean += unseen / nf / 3.0;
        for a in 0..3 { action_mean[a] += act[a] / nf / 3.0; }
    }
    assert!(draw_mean <= MEASURED_DRAW_RATE + 0.10, "mean draw rate {draw_mean:.3}");
    assert!(unseen_mean <= MEASURED_UNSEEN_BLOCK + 0.05, "mean unseen block share {unseen_mean:.3}");
    for (a, s) in action_mean.iter().enumerate() { assert!(*s >= 0.02, "action {a} mean share {s}"); }
}
