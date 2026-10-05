use bitp::{sim::PlayerIntentSim, util::Seq};
use std::collections::VecDeque;

const QUEUE_MAX: usize = 8;

pub struct PlayerIntentQueue {
    queue: VecDeque<(Seq, PlayerIntentSim)>,
    last_sim: PlayerIntentSim,

    /// Newest sequence that arrived
    latest_seq: Option<Seq>,

    /// The sequence number the simulation most recently simulated.
    /// Used to echo it on snapshots, and the client will use it for
    /// reconciliation
    simed_seq: Seq,
}

impl PlayerIntentQueue {
    pub fn new() -> Self {
        PlayerIntentQueue {
            queue: VecDeque::new(),
            last_sim: PlayerIntentSim::default(),
            latest_seq: None,
            simed_seq: Seq(0),
        }
    }

    pub fn queue(&mut self, seq: Seq, sim_intent: PlayerIntentSim) {
        if self
            .latest_seq
            .is_some_and(|latest| !seq.is_newer_than(latest))
        {
            return; // duplicate or stale
        }
        self.latest_seq = Some(seq);

        if self.queue.len() >= QUEUE_MAX {
            self.queue.pop_front();
        }
        self.queue.push_back((seq, sim_intent));
    }

    /// Pops the next queued intent for this tick and returns it.
    /// If the queue is empty it repeats the last intent with the edge fields
    /// reset so a lost packet can't retrigger them
    pub fn next(&mut self) -> PlayerIntentSim {
        match self.queue.pop_front() {
            Some((simed_seq, sim_intent)) => {
                self.simed_seq = simed_seq;
                self.last_sim = sim_intent;
            }
            None => self.last_sim = self.last_sim.held_only(),
        }
        self.last_sim
    }

    pub fn simed_seq(&self) -> Seq {
        self.simed_seq
    }
}
