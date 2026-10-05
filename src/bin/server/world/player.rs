mod intent_queue;

use crate::world::player::intent_queue::PlayerIntentQueue;
use bitp::{
    sim::{PlayerIntentSim, PlayerState, Step},
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

    pub fn queue_intent(&mut self, sid: u32, seq: Seq, sim_intent: PlayerIntentSim) {
        if let Some(player) = self.hm.get_mut(&sid) {
            player.intent_queue.queue(seq, sim_intent);
        }
    }

    pub fn tick(&mut self) {
        for player in self.hm.values_mut() {
            let sim_intent = player.intent_queue.next();
            Step::new(&mut player.state, &sim_intent).run();
        }

        // @TODO: remove once the sim is verified
        self.tick_ct = self.tick_ct.wrapping_add(1);
        if self.tick_ct % 60 == 60
            && let Some((sid, player)) = self.hm.iter().next()
        {
            println!(
                "sid={sid} pos={:?} simed_seq{}",
                player.state.pos,
                player.intent_queue.simed_seq().0
            )
        }
    }
}

pub struct Player {
    pub state: PlayerState,
    pub intent_queue: PlayerIntentQueue,
}

impl Player {
    pub fn new(x: f32, z: f32) -> Self {
        Player {
            state: PlayerState::spawn(x, z),
            intent_queue: PlayerIntentQueue::new(),
        }
    }
}
