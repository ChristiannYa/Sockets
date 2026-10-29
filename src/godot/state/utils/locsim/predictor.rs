use crate::{
    pkt::Seq,
    qua::player as pqua,
    sim::{PlayerIntentSim, PlayerState},
    util::now,
};
use std::collections::VecDeque;

/// Max entries kept.
///
/// It must stay under 128, otherwise the circular [`Seq::is_newer_than`]
/// compare against an ack older than the oldest entry flips
const CAP: usize = 64;

/// Largest position error (meters) that quantization alone can explain.
///
/// The server's position arrives rounded to the wire grid while the client
/// predicts with the raw floats, so the two differ even when they agree.
/// The worst case is a point in the middle of a grid cell, half the cell's
/// diagonal away from the nearest grid point, so anything at or below this
/// is rounding noise. Anything above it is a real disagreement (a lost
/// intent, a dry tick, a jump landing on a different tick) and gets
/// corrected.
///
/// Derived from the position quantizers, so it follows them if their steps
/// change. It is not widened for gameplay effects: if corrections turn out
/// to be too frequent, a margin goes here
pub(super) fn pos_tolerance() -> f32 {
    let diag = pqua::LOC_HOR
        .x
        .step()
        .hypot(pqua::intent::Y.step())
        .hypot(pqua::LOC_HOR.z.step());
    diag / 2.0
}

/// Distance between the predicted position and the one the server sent
pub(super) fn pos_err(predicted: &PlayerState, server: [f32; 3]) -> f32 {
    let d = |i: usize| predicted.pos[i] - server[i];
    (d(0).powi(2) + d(1).powi(2) + d(2).powi(2)).sqrt()
}

/// One predicted tick
struct Entry {
    seq: Seq,

    /// The bool form that was actually stepped, so a replay doesn't recompute
    /// `fired` against a different `prev`
    intent: PlayerIntentSim<bool>,

    /// The full state after the step
    after: PlayerState,
}

/// Remembers what the client predicted so that a snapshot can be checked
/// against it and the unacked inputs replayed (see [`PlayerIntentSim`])
pub(super) struct Predictor {
    /// Oldest first, which is also seq order
    ring: VecDeque<Entry>,
}

impl Predictor {
    pub(super) fn new() -> Predictor {
        Predictor { ring: VecDeque::with_capacity(CAP) }
    }

    /// Stores a stepped intent.
    ///
    /// At [`CAP`] the oldest entry is dropped
    pub(super) fn record(
        &mut self,
        seq: Seq,
        intent: PlayerIntentSim<bool>,
        after: PlayerState,
    ) {
        debug_assert!(
            self.ring
                .back()
                .is_none_or(|last| seq.is_newer_than(last.seq)),
            "[{}] predictor records must come in seq order",
            now()
        );

        if self.ring.len() == CAP {
            self.ring.pop_front();
        }
        self.ring.push_back(Entry { seq, intent, after });
    }

    /// Handles a server ack of everything up to and including `simed_seq`.
    ///
    /// Drops those entries and returns the state predicted right after
    /// `simed_seq`, otherwise [`None`] if there is no entry for it (it was
    /// never recored, or it already fell out of the ring).
    /// What remains is exactly the unacked intents, oldest first, for the
    /// replay
    pub(super) fn ack(&mut self, simed_seq: Seq) -> Option<PlayerState> {
        let predicted = self
            .ring
            .iter()
            .find(|en| en.seq == simed_seq)
            .map(|en| en.after);

        self.ring.retain(|en| en.seq.is_newer_than(simed_seq));

        predicted
    }
}
