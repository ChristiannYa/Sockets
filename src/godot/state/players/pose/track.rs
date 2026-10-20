mod history;

use super::{PoseSample, RawPose};
use crate::bits::PacketSchema;
use history::PoseHistory;

/// One player's pose: the last-known raw fields, and the history decoded
/// from them
pub struct PoseTrack {
    raw: RawPose,
    pub history: PoseHistory,
}

impl PoseTrack {
    pub fn new(history_cap: usize) -> Self {
        PoseTrack {
            raw: RawPose::default(),
            history: PoseHistory::new(history_cap),
        }
    }

    /// Merge what one record carried into this player's last-known pose fields
    /// and, once every field is known, add the decoded pose to the history at
    /// `t_s`.
    ///
    /// A record may carry only some of the fields, so the others are kept from
    /// earlier records.
    ///
    /// ```text
    /// player 3, nothing known yet
    ///   record: x=10 y=0 z=5 yaw=1 crouch=0 -> all known: pose added
    ///   record: x=12                        -> x replaced, rest kept: pose added
    ///
    /// player 7, nothing known yet
    ///   record: x=4                         -> y, z, yaw, crouch unknown:
    ///                                          nothing added
    /// ```
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
