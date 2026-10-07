//! Arena. WS2 owns the full implementation; WS1 only provides the minimal flat arena the shared test helpers need.

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
}

impl Arena {
    pub fn flat(width: f32) -> Arena {
        Arena { width, obstacles: Vec::new() }
    }
}
