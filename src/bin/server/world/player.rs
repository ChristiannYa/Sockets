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
        self.tick_ct = self.tick_ct.wrapping_add(1);

        for (sid, player) in self.hm.iter_mut() {
            let before = player.intent_queue.simed_seq().0;
            let sim_intent = player.intent_queue.next();
            Step::new(&mut player.state, &sim_intent).run();
            let after = player.intent_queue.simed_seq().0;

            // @TODO: remove once the sim intent is verified
            if after != before {
                println!(
                    "tick={} sid={sid} simed_seq={after} pos={:?}",
                    self.tick_ct, player.state.pos
                );
            } else if self.tick_ct.is_multiple_of(60) {
                println!(
                    "tick={} sid={sid} idle-simed_seq={after} pos={:?}",
                    self.tick_ct, player.state.pos
                )
            }
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
