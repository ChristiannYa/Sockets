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
    tick_ct: u32,
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

        // @TODO: remove once the sim is verified
        self.tick_ct = self.tick_ct.wrapping_add(1);
        if self.tick_ct % 60 == 60
            && let Some((sid, player)) = self.hm.iter().next()
        {
            println!(
                "sid={sid} pos={:?} simed_seq{}",
                player.state.pos,
                player.inp_queue.simed_seq().0
            )
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
