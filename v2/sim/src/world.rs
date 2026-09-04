//! Arena grid: blocked tiles, spawns, reachability.
use crate::config::ArenaConfig;
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Arena {
    w: usize,
    h: usize,
    blocked: Vec<bool>,
    spawns: [(f32, f32); 2],
}

impl Arena {
    pub fn generate(seed: u64, cfg: &ArenaConfig) -> Arena {
        let w = cfg.arena_w as usize;
        let h = cfg.arena_h as usize;
        let mut rng = Rng::new(seed ^ 0xA4E4_A000_0000_0001);
        let spawns = [(3.5, h as f32 / 2.0 + 0.5), (w as f32 - 3.5, h as f32 / 2.0 + 0.5)];
        let mut blocked = vec![false; w * h];
        let n_obst = 14 + (rng.next_u64() % 10) as usize;
        for _ in 0..n_obst {
            let bw = 1 + (rng.next_u64() % 3) as usize;
            let bh = 1 + (rng.next_u64() % 3) as usize;
            let bx = (rng.next_u64() % w as u64) as usize;
            let by = (rng.next_u64() % h as u64) as usize;
            for y in by..(by + bh).min(h) {
                for x in bx..(bx + bw).min(w) {
                    // Keep the centre band (used by duels) and spawn zones open.
                    let centre = (14..34).contains(&x) && (13..19).contains(&y);
                    let near_spawn = (x < 7 || x >= w - 7) && (h / 2 - 3..h / 2 + 4).contains(&y);
                    if !centre && !near_spawn {
                        blocked[y * w + x] = true;
                    }
                }
            }
        }
        let mut a = Arena { w, h, blocked, spawns };
        if !a.reachable(spawns[0], spawns[1]) {
            for b in a.blocked.iter_mut() {
                *b = false;
            }
        }
        a
    }
