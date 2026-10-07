mod intent_queue;

use crate::world::player::intent_queue::PlayerIntentQueue;
use bitp::{
    sim::{Collision, PlayerIntentSim, PlayerState, Step},
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

    pub fn queue_intent(
        &mut self,
        sid: u32,
        seq: Seq,
        sim_intent: PlayerIntentSim,
    ) {
        if let Some(player) = self.hm.get_mut(&sid) {
            player.intent_queue.queue(seq, sim_intent);
        }
    }

    pub fn tick(&mut self) {
        self.step_all();
        self.resolve_collisions()
    }

    fn step_all(&mut self) {
        for player in self.hm.values_mut() {
            let sim_intent = player.intent_queue.next();
            Step::new(&mut player.state, &sim_intent).run();
        }
    }

    fn resolve_collisions(&mut self) {
        // Sorted by sid, because HashMap iteration order is random
        let mut states: Vec<(u32, &mut PlayerState)> = self
            .hm
            .iter_mut()
            .map(|(sid, p)| (*sid, &mut p.state))
            .collect();
        states.sort_unstable_by_key(|(sid, _)| *sid);

        Collision::new(
            states
                .into_iter()
                .map(|(_, p_s)| p_s)
                .collect(),
        )
        .resolve();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u32, &Player)> {
        self.hm.iter()
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
