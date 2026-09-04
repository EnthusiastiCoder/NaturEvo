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

    fn tile_blocked(&self, tx: i32, ty: i32) -> bool {
        if tx < 0 || ty < 0 || tx >= self.w as i32 || ty >= self.h as i32 {
            return true;
        }
        self.blocked[ty as usize * self.w + tx as usize]
    }

    pub fn is_blocked(&self, x: f32, y: f32) -> bool {
        if !(x >= 0.0 && y >= 0.0) {
            return true;
        }
        self.tile_blocked(x.floor() as i32, y.floor() as i32)
    }

    /// Whether a circle of the given radius at (x, y) overlaps a blocked tile or the border.
    pub fn circle_blocked(&self, x: f32, y: f32, r: f32) -> bool {
        let x0 = (x - r).floor() as i32;
        let x1 = (x + r).floor() as i32;
        let y0 = (y - r).floor() as i32;
        let y1 = (y + r).floor() as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if self.tile_blocked(tx, ty) {
                    let cx = x.clamp(tx as f32, tx as f32 + 1.0);
                    let cy = y.clamp(ty as f32, ty as f32 + 1.0);
                    let (dx, dy) = (x - cx, y - cy);
                    if dx * dx + dy * dy < r * r {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn spawns(&self) -> [(f32, f32); 2] {
        self.spawns
    }
