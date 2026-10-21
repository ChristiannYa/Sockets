use crate::world::player::edges;
use bitp::{
    logt,
    pkt::Seq,
    sim::{PlayerIntentSim, PlayerIntentSimEdges},
};
use std::collections::VecDeque;

const QUEUE_MAX: usize = 8;

/// Ticks in a row without a queued intent before it gets a log line
const DRY_WARN_TICKS: u32 = 3;

pub struct PlayerIntentQueue {
    queue: VecDeque<(Seq, PlayerIntentSim<u8>)>,
    last_sim: PlayerIntentSim<bool>,

    /// Newest sequence that arrived
    latest_seq: Option<Seq>,

    /// The sequence number the simulation most recently simulated.
    /// Used to echo it on snapshots, and the client will use it for
    /// reconciliation
    simed_seq: Seq,

    /// Edge counters from the intent the simulation most recently simulated.
    /// None until the first intent, which only seeds it, so a player whose
    /// counters aren't 0 can't fire phantom events.
    simed_edge_cts: Option<PlayerIntentSimEdges<u8>>,

    /// Ticks in a row where the queue was empty
    dry_ticks: u32,
}

impl PlayerIntentQueue {
    pub fn new() -> Self {
        PlayerIntentQueue {
            queue: VecDeque::new(),
            last_sim: PlayerIntentSim::<bool>::default(),
            latest_seq: None,
            simed_seq: Seq(0),
            simed_edge_cts: None,
            dry_ticks: 0,
        }
    }

    pub fn queue(&mut self, seq: Seq, cmd: PlayerIntentSim<u8>) {
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
        self.queue.push_back((seq, cmd));
    }

    /// Pops the next queued intent for this tick and returns it.
    /// If the queue is empty it repeats the last intent with the edge fields
    /// reset so a lost packet can't retrigger them
    pub fn next(&mut self) -> PlayerIntentSim<bool> {
        match self.queue.pop_front() {
            Some((simed_seq, cmd)) => {
                if self.dry_ticks >= DRY_WARN_TICKS {
                    logt!(
                        "intents flowing again after {} dry ticks",
                        self.dry_ticks
                    );
                }
                self.dry_ticks = 0;

                let edges = match self.simed_edge_cts {
                    Some(prev) => edges::diffs(cmd.edges, prev),
                    None => PlayerIntentSimEdges::default(),
                };
                self.simed_edge_cts = Some(cmd.edges);

                self.simed_seq = simed_seq;
                self.last_sim =
                    PlayerIntentSim::<bool> { held: cmd.held, edges };
            }
            None => {
                self.dry_ticks += 1;
                if self.dry_ticks == DRY_WARN_TICKS {
                    logt!(
                        "intent queue ran dry for {DRY_WARN_TICKS} ticks, repeating last input"
                    );
                }

                self.last_sim = self.last_sim.held_only();
            }
        }
        self.last_sim
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitp::sim::PlayerIntentSimHeld;

    fn cmd(jump_ct: u8) -> PlayerIntentSim<u8> {
        PlayerIntentSim::<u8> {
            held: PlayerIntentSimHeld::default(),
            edges: PlayerIntentSimEdges { jump: jump_ct },
        }
    }

    #[test]
    fn first_intent_only_seeds() {
        let mut q = PlayerIntentQueue::new();
        // Not 0 on purpose: a joiner's counter can start anywhere
        q.queue(Seq(0), cmd(5));
        q.queue(Seq(1), cmd(5));

        assert!(!q.next().edges.jump);
        assert!(!q.next().edges.jump);
    }

    #[test]
    fn counter_change_fires_once() {
        let mut q = PlayerIntentQueue::new();
        q.queue(Seq(0), cmd(1));
        q.queue(Seq(1), cmd(2));
        q.queue(Seq(2), cmd(2));

        let jumps: Vec<bool> = (0..3).map(|_| q.next().edges.jump).collect();
        assert_eq!(jumps, [false, true, false]);
    }
}
