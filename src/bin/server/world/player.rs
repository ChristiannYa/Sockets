mod intent_queue;

use crate::world::{
    self, logic::color::Hsv, player::intent_queue::PlayerIntentQueue,
};
use bitp::sim::PlayerState;
use std::collections::HashMap;

pub type PlayersHM = HashMap<u32, Player>;

pub struct Players {
    hm: PlayersHM,
}

impl Players {
    pub fn new() -> Self { Players { hm: HashMap::new() } }

    pub fn iter(&self) -> impl Iterator<Item = (&u32, &Player)> {
        self.hm.iter()
    }

    pub fn by_sid_mut_ref(&mut self, sid: u32) -> Option<&mut Player> {
        self.hm.get_mut(&sid)
    }

    pub fn spawn(&mut self, sid: u32, x: f32, z: f32, color: Hsv) {
        self.hm.insert(sid, Player::new(x, z, color));
    }

    pub fn sim_tick(&mut self) { world::sim::players::tick(&mut self.hm); }
}

impl Default for Players {
    fn default() -> Self { Self::new() }
}

pub struct Player {
    pub state: PlayerState,
    pub intent_queue: PlayerIntentQueue,
    pub color: Hsv,
}

impl Player {
    pub fn new(x: f32, z: f32, color: Hsv) -> Self {
        Player {
            state: PlayerState::spawn(x, z),
            intent_queue: PlayerIntentQueue::new(),
            color,
        }
    }
}
