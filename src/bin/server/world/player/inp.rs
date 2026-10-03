use bitp::{sim::Input, util::Seq};
use std::collections::VecDeque;

const QUEUE_MAX: usize = 8;

pub struct PlayerInput {
    queue: VecDeque<(Seq, Input)>,
    last: Input,

    /// Newest sequence that arrived
    latest_seq: Option<Seq>,

    /// The sequence number the simulation most recently simulated.
    /// Used to echo it on snapshots, and the client will use it for
    /// reconciliation
    sim_seq: Seq,
}

impl PlayerInput {
    pub fn new() -> Self {
        PlayerInput {
            queue: VecDeque::new(),
            last: Input::default(),
            latest_seq: None,
            sim_seq: Seq(0),
        }
    }

    pub fn push(&mut self, seq: Seq, inp: Input) {
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
        self.queue.push_back((seq, inp));
    }

    /// Pops the next queued input for this tick.
    /// If the queue is empty, repeats the last input with `jump` cleared so a
    /// lost packet can't re-trigger a jump.
    pub fn next(&mut self) -> Input {
        match self.queue.pop_front() {
            Some((seq, inp)) => {
                // Consume queued input
                self.sim_seq = seq;
                self.last = inp;
            }
            None => self.last = self.last.held_only(),
        }
        self.last
    }

    pub fn applied_seq(&self) -> Seq {
        self.sim_seq
    }
}
