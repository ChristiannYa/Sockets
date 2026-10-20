mod history;

use super::{PoseSample, RawPose};
use crate::bits::PacketSchema;
use history::PoseHistory;

/// One player's pose: the last-known raw fields, and the history decoded
/// from them
pub struct PoseTrack {
    raw: RawPose,
    history: PoseHistory,
}

impl PoseTrack {
    pub fn new(history_cap: usize) -> Self {
        PoseTrack {
            raw: RawPose::default(),
            history: PoseHistory::new(history_cap),
        }
    }

    pub fn at(&self, t_s: f64) -> Option<PoseSample> { self.history.at(t_s) }

    pub fn latest(&self) -> Option<PoseSample> { self.history.latest() }

    /// Merge what one record carried into this player's last-known pose fields
    /// and, once every field is known, add the decoded pose to the history at
    /// `t_s`.
    pub fn apply(
        &mut self,
        delta: &RawPose,
        schema: &PacketSchema,
        t_s: f64,
    ) {
        self.raw.merge(delta);
        if let Some(pose) = self.raw.decode(schema) {
            self.history.push(t_s, pose);
        }
    }
}
