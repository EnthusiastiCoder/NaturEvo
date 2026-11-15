//! Arena: flat ground, low obstacle boxes standing on it, walls at both ends.
use crate::config::ARENA_WIDTH;
use crate::rng::Rng;

/// Visual/material setting of an arena (data only; no physics effect).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Scrapland,
    Jungle,
}

impl Biome {
    pub fn name(&self) -> &'static str {
        match self {
            Biome::Scrapland => "scrapland",
            Biome::Jungle => "jungle",
        }
    }
}

/// Axis-aligned box on the ground, centred at `x`, `w` wide, `h` high.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub x: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug)]
pub struct Arena {
    pub width: f32,
    pub obstacles: Vec<Obstacle>,
    pub biome: Biome,
}

impl Arena {
    pub fn flat(width: f32) -> Arena {
        Arena { width, obstacles: Vec::new(), biome: Biome::Scrapland }
    }

    /// Deterministic arena from a seed: 2 to 3 low boxes (0.6 to 1.5 m) in fixed slots so spawns stay clear.
    pub fn generate(seed: u64) -> Arena {
        let mut rng = Rng::new(seed ^ 0xA7E4_A000);
        let slots: [(f32, f32); 3] = [(1.0, 3.5), (5.8, 8.2), (10.5, 13.0)];
        let skip_middle = rng.f32() < 0.4;
        let mut obstacles = Vec::new();
        for (i, (lo, hi)) in slots.iter().enumerate() {
            let w = rng.range(0.6, 1.2);
            let h = rng.range(0.6, 1.5);
            let x = rng.range(lo + w / 2.0, hi - w / 2.0);
            if i == 1 && skip_middle {
                continue;
            }
            obstacles.push(Obstacle { x, w, h });
        }
        let biome = if rng.f32() < 0.8 { Biome::Scrapland } else { Biome::Jungle };
        Arena { width: ARENA_WIDTH, obstacles, biome }
    }

    /// Clamp a robot root x to the arena walls.
    pub fn clamp_x(&self, x: f32) -> f32 {
        x.clamp(0.5, self.width - 0.5)
    }
}
