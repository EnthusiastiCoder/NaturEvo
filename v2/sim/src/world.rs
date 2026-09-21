//! Arena grid: blocked tiles, spawns, reachability.
use crate::config::ArenaConfig;
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Arena {
    w: usize,
    h: usize,
    blocked: Vec<bool>,
    /// Chebyshev tile distance to the nearest blocked tile or border (0 = blocked), capped at 255.
    clear: Vec<u8>,
    padded: Vec<u8>,
    spawns: [(f32, f32); 2],
}

/// Padding (tiles) of blocked border around the grid in `Arena::padded`.
const PAD: usize = 20;

fn ray_mask_scalar(padded: &[u8], pw: i32, ox: i32, oy: i32, sx: i32, sy: i32) -> u32 {
    let mut mask = 0u32;
    for r in 1..16i32 {
        let (tx, ty) = ((ox + sx * r) >> 16, (oy + sy * r) >> 16);
        mask |= (padded[(ty * pw + tx) as usize] as u32) << r;
    }
    mask
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn ray_mask_avx2(padded: &[u8], pw: i32, ox: i32, oy: i32, sx: i32, sy: i32) -> u32 {
    use std::arch::x86_64::*;
    let (vox, voy) = (_mm256_set1_epi32(ox), _mm256_set1_epi32(oy));
    let (vsx, vsy) = (_mm256_set1_epi32(sx), _mm256_set1_epi32(sy));
    let (vpw, lo8) = (_mm256_set1_epi32(pw), _mm256_set1_epi32(0xFF));
    let zero = _mm256_setzero_si256();
    let mut mask = 0u32;
    for half in 0..2 {
        let r = _mm256_setr_epi32(1, 2, 3, 4, 5, 6, 7, 8);
        let r = _mm256_add_epi32(r, _mm256_set1_epi32(8 * half));
        let tx = _mm256_srai_epi32(_mm256_add_epi32(vox, _mm256_mullo_epi32(vsx, r)), 16);
        let ty = _mm256_srai_epi32(_mm256_add_epi32(voy, _mm256_mullo_epi32(vsy, r)), 16);
        let idx = _mm256_add_epi32(_mm256_mullo_epi32(ty, vpw), tx);
        // SAFETY: indices stay inside `padded` (PAD covers the largest offset; the buffer has
        // 4 spare bytes for the 32-bit gather of the last element).
        let g = _mm256_i32gather_epi32::<1>(padded.as_ptr() as *const i32, idx);
        let nz = _mm256_cmpgt_epi32(_mm256_and_si256(g, lo8), zero);
        mask |= (_mm256_movemask_ps(_mm256_castsi256_ps(nz)) as u32) << (8 * half);
    }
    mask << 1
}

#[inline(always)]
fn ray_mask(padded: &[u8], pw: i32, ox: i32, oy: i32, sx: i32, sy: i32) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: avx2 detected at runtime.
            return unsafe { ray_mask_avx2(padded, pw, ox, oy, sx, sy) };
        }
    }
    ray_mask_scalar(padded, pw, ox, oy, sx, sy)
}

impl Arena {
    pub fn generate(seed: u64, cfg: &ArenaConfig) -> Arena {
        let w = cfg.arena_w as usize;
        let h = cfg.arena_h as usize;
        let mut rng = Rng::new(seed ^ 0xA4E4_A000_0000_0001);
        let spawns = [(w as f32 / 2.0 - 6.5, h as f32 / 2.0 + 0.5), (w as f32 / 2.0 + 6.5, h as f32 / 2.0 + 0.5)];
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
        let mut a = Arena { w, h, blocked, clear: Vec::new(), padded: Vec::new(), spawns };
        if !a.reachable(spawns[0], spawns[1]) {
            for b in a.blocked.iter_mut() {
                *b = false;
            }
        }
        a.clear = a.compute_clear();
        let pw = w + 2 * PAD;
        let mut padded = vec![1u8; pw * (h + 2 * PAD) + 4];
        for y in 0..h {
            for x in 0..w {
                padded[(y + PAD) * pw + x + PAD] = a.blocked[y * w + x] as u8;
            }
        }
        a.padded = padded;
        a
    }

    #[inline(always)]
    fn tile_blocked(&self, tx: i32, ty: i32) -> bool {
        if tx < 0 || ty < 0 || tx >= self.w as i32 || ty >= self.h as i32 {
            return true;
        }
        self.blocked[ty as usize * self.w + tx as usize]
    }

    /// Per tile: 0 if blocked, else max(1, ceil(g)) with g the Euclidean gap from the tile's
    /// rectangle to the nearest blocked tile (or the outside), capped at 255. Every point
    /// within distance < g of a point in the tile is free.
    fn compute_clear(&self) -> Vec<u8> {
        let (w, h) = (self.w as i32, self.h as i32);
        let mut d = vec![0u8; self.w * self.h];
        let list: Vec<(i32, i32)> = (0..w * h).filter(|&i| self.blocked[i as usize]).map(|i| (i % w, i / w)).collect();
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if d[i] == 0 { continue; }
                let m = at(&d, x - 1, y).min(at(&d, x, y - 1)).min(at(&d, x - 1, y - 1)).min(at(&d, x + 1, y - 1));
                d[i] = d[i].min(m.saturating_add(1));
            }
        }
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let i = (y * w + x) as usize;
                if d[i] == 0 { continue; }
                let m = at(&d, x + 1, y).min(at(&d, x, y + 1)).min(at(&d, x + 1, y + 1)).min(at(&d, x - 1, y + 1));
                d[i] = d[i].min(m.saturating_add(1));
            }
        }
        d
    }

    /// Distance along the ray (x, y) + t*(dx, dy) to the first blocked sample taken at
    /// t = 1, 2, 3, ... strictly below `range`; `range` if none. Samples that provably
    /// cannot be blocked (inside the clearance of a free tile) are skipped, so the result
    /// equals plain unit stepping. The step index stays an integer so the loop-carried
    /// dependency does not go through float conversions.
    pub fn ray_dist(&self, x: f32, y: f32, dx: f32, dy: f32, range: f32) -> f32 {
        let (wi, hi) = (self.w as u32, self.h as u32);
        let mut i = 1i32;
        while (i as f32) < range {
            let r = i as f32;
            let (px, py) = (x + dx * r, y + dy * r);
            // Shift by one so truncation floors points in [-1, 0) to tile -1; any negative
            // tile (or NaN, which casts to 0 -> -1) fails the unsigned bounds check.
            let (tx, ty) = ((px + 1.0) as i32 - 1, (py + 1.0) as i32 - 1);
            if (tx as u32) >= wi || (ty as u32) >= hi {
                return r;
            }
            let c = self.clear[ty as usize * self.w + tx as usize] as i32;
            if c == 0 {
                return r;
            }
            // Points within distance < c - 1 lie in free tiles: skip the next c - 2 samples.
            i += 1 + (c - 2).max(0);
        }
        range
    }

    #[inline(always)]
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

    pub fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    pub fn tiles(&self) -> &[bool] {
        &self.blocked
    }

    pub fn reachable(&self, a: (f32, f32), b: (f32, f32)) -> bool {
        if self.is_blocked(a.0, a.1) || self.is_blocked(b.0, b.1) {
            return false;
        }
        let start = (a.0.floor() as i32, a.1.floor() as i32);
        let goal = (b.0.floor() as i32, b.1.floor() as i32);
        let mut seen = vec![false; self.w * self.h];
        let mut stack = vec![start];
        seen[start.1 as usize * self.w + start.0 as usize] = true;
        while let Some((x, y)) = stack.pop() {
            if (x, y) == goal {
                return true;
            }
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                if !self.tile_blocked(nx, ny) {
                    let i = ny as usize * self.w + nx as usize;
                    if !seen[i] {
                        seen[i] = true;
                        stack.push((nx, ny));
                    }
                }
            }
        }
        false
    }
}
