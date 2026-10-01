use crate::sim::consts::{FLOOR_Y, HEIGHT};

#[derive(Debug, Clone, Copy)]
pub struct PlayerState {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub height: f32,
    pub is_on_floor: bool,
}

impl PlayerState {
    pub fn spawn(x: f32, z: f32) -> Self {
        PlayerState {
            pos: [x, FLOOR_Y, z],
            vel: [0.0; 3],
            height: HEIGHT,
            is_on_floor: true,
        }
    }
}
