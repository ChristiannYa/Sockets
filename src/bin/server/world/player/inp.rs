use bitp::{sim::PlayerInpSim, util::Seq};
use std::collections::VecDeque;

const QUEUE_MAX: usize = 8;

pub struct PlayerInpQueue {
    queue: VecDeque<(Seq, PlayerInpSim)>,
    last_sim: PlayerInpSim,

    /// Newest sequence that arrived
    latest_seq: Option<Seq>,

    /// The sequence number the simulation most recently simulated.
    /// Used to echo it on snapshots, and the client will use it for
    /// reconciliation
    simed_seq: Seq,
}

impl PlayerInpQueue {
    pub fn new() -> Self {
        PlayerInpQueue {
            queue: VecDeque::new(),
            last_sim: PlayerInpSim::default(),
            latest_seq: None,
            simed_seq: Seq(0),
        }
    }

    pub fn push(&mut self, seq: Seq, sim_inp: PlayerInpSim) {
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
        self.queue.push_back((seq, sim_inp));
    }

    /// Pops the next queued input for this tick and returns it.
    /// If the queue is empty it repeats the last input with the edge fields
    /// reset so a lost packet can't retrigger them
    pub fn next(&mut self) -> PlayerInpSim {
        match self.queue.pop_front() {
            Some((simed_seq, sim_inp)) => {
                self.simed_seq = simed_seq;
                self.last_sim = sim_inp;
            }
            None => self.last_sim = self.last_sim.held_only(),
        }
        self.last_sim
    }

    pub fn simed_seq(&self) -> Seq {
        self.simed_seq
    }
}
