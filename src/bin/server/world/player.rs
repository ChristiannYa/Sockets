mod inp_queue;

use crate::world::player::inp_queue::PlayerInpQueue;
use bitp::{
    sim::{PlayerInpSim, PlayerState, Step},
    util::Seq,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Players {
    hm: HashMap<u32, Player>,
}

impl Players {
    pub fn spawn(&mut self, sid: u32, x: f32, z: f32) {
        self.hm.insert(sid, Player::new(x, z));
    }

    pub fn push_inp(&mut self, sid: u32, seq: Seq, sim_inp: PlayerInpSim) {
        if let Some(player) = self.hm.get_mut(&sid) {
            player.inp_queue.push(seq, sim_inp);
        }
    }

    pub fn tick(&mut self) {
        for player in self.hm.values_mut() {
            let sim_inp = player.inp_queue.next();
            Step::new(&mut player.state, &sim_inp).run();
        }
    }
    pub fn remove(&mut self, sid: u32) {
        self.hm.remove(&sid);
    }
}

pub struct Player {
    pub state: PlayerState,
    pub inp_queue: PlayerInpQueue,
}

impl Player {
    pub fn new(x: f32, z: f32) -> Self {
        Player {
            state: PlayerState::spawn(x, z),
            inp_queue: PlayerInpQueue::new(),
        }
    }
}
