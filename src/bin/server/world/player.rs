mod inp;

use bitp::sim::PlayerState;

use crate::world::player::inp::PlayerInput;

pub struct Player {
    pub state: PlayerState,
    pub inp: PlayerInput,
}

impl Player {
    pub fn new(x: f32, z: f32) -> Self {
        Player {
            state: PlayerState::spawn(x, z),
            inp: PlayerInput::new(),
        }
    }
}
