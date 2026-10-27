mod history;

use super::{PoseSample, RawPose};
use crate::{
    bits::PacketSchema, godot::state::RENDER_DELAY_S, sim::consts::SS_INTV_S,
};
use history::PoseHistory;

/// Samples kept per player. Enough to cover the render delay, +2 for the pair
/// that brackets the render time.
///
/// The capacity is a count and the delay is a time, so compare them by the
/// time the list spans: `(cap - 1) * SS_INTV_S` (N samples span N - 1 gaps).
///
/// ```text
/// 60hz, delay 100ms:
/// cap 8 -> spans 7 * 16.7 = 116.7ms  (covers the delay)
/// cap 4 -> spans 3 * 16.7 =  50.0ms  (too small)
/// ```
///
/// If the list spans less than [`RENDER_DELAY_S`], the render time falls
/// before the oldest sample, so `PoseHistory::at` returns that oldest pose
/// as is, with no interpolation, and the player would look stuck.
const HISTORY_CAP: usize = ((RENDER_DELAY_S / SS_INTV_S).ceil() as usize) + 2;

/// One player's pose: the last-known raw fields, and the history decoded
/// from them
pub struct PoseTrack {
    raw: RawPose,
    history: PoseHistory,
}

impl PoseTrack {
    pub fn new() -> Self {
        PoseTrack {
            raw: RawPose::default(),
            history: PoseHistory::new(HISTORY_CAP),
        }
    }

    pub fn at(&self, t_s: f64) -> Option<PoseSample> { self.history.at(t_s) }

    pub fn latest(&self) -> Option<PoseSample> { self.history.latest() }

    /// Merge what one record carried into this player's last-known pose fields
    /// and, once every field is known, add the decoded pose to the history at
    /// `t_s`.
    pub fn apply(&mut self, delta: &RawPose, t_s: f64) {
        self.raw.merge(delta);
        if let Some(pose) = self.raw.decode() {
            self.history.push(t_s, pose);
        }
    }
}
