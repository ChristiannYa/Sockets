use std::collections::VecDeque;

use crate::{
    pkt::Seq,
    sim::{PlayerIntentSim, PlayerState},
    util::now,
};

/// Max entries kept.
///
/// It must stay under 128, otherwise the circular [`Seq::is_newer_than`] compare
/// against an ack older than the oldest entry flips
const CAP: usize = 64;

/// One predicted tick
pub(super) struct PredictorEntry {
    pub(super) seq: Seq,

    /// The bool form that was actually stepped, so a replay doesn't recompute
    /// `fired` against a different `prev`
    pub(super) intent: PlayerIntentSim<bool>,

    /// The full state after the step
    pub(super) after: PlayerState,
}

/// Remembers what the client predicted so that a snapshot can be checked
/// against it and the unacked inputs replayed (see [`PlayerIntentSim`])
pub(super) struct Predictor {
    /// Oldest first, which is also seq order
    ring: VecDeque<PredictorEntry>,
}

impl Predictor {
    pub(super) fn new() -> Predictor {
        Predictor { ring: VecDeque::with_capacity(CAP) }
    }

    /// Stores a stepped intent.
    ///
    /// At [`CAP`] the oldest entry is dropped
    pub(super) fn record(&mut self, entry: PredictorEntry) {
        debug_assert!(
            self.ring
                .back()
                .is_none_or(|last| entry.seq.is_newer_than(last.seq)),
            "[{}] predictor records must come in seq order",
            now()
        );

        if self.ring.len() == CAP {
            self.ring.pop_front();
        }
        self.ring.push_back(entry);
    }
}
