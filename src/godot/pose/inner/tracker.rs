mod records;

use super::{history::PoseHistory, raw::RawPose, sample::PoseSample};
use crate::bits::PacketSchema;
use records::records;
use std::collections::HashMap;

struct Track {
    raw: RawPose,
    history: PoseHistory,
}

/// Per-player pose state: merges delta records, decodes once on arrival, and
/// keeps a history to interpolate from.
pub struct PoseTracker {
    schema: PacketSchema,
    history_cap: usize,
    tracks: HashMap<u32, Track>,
}

impl PoseTracker {
    pub fn new(schema: PacketSchema, history_cap: usize) -> Self {
        PoseTracker { schema, history_cap, tracks: HashMap::new() }
    }

    /// Reads every record in `body` and updates each player's pose history.
    ///
    /// `body` is a records buffer (layout in [`records::records`]).
    /// `recv_t_s` is the time of the poses decoded from it. [`Self::sample`]
    /// looks poses up on that same clock.
    ///
    /// For each record:
    /// 1. The pose fields it carries are merged into that player's last-known
    ///    fields, so a partial record only updates what it carries;
    /// 2. Once every pose field is known, the pose is decoded and added to the
    ///    player's history.
    ///
    /// Malformed records and records without a session id are skipped. A
    /// player has no history until every pose field has been received at least
    /// once.
    ///
    /// ```text
    /// sid=3 x=10 y=0 z=5 yaw=1 crouch=0  -> all known:
    ///                                         pose pushed
    ///
    /// sid=3 x=12                         -> merged (x=12, rest kept):
    ///                                         pose pushed
    ///
    /// sid=7 x=4                          -> y, z, yaw, crouch unknown:
    ///                                         nothing pushed
    /// ```
    pub fn ingest(&mut self, body: &[u8], recv_t_s: f64) {
        let history_cap = self.history_cap;

        for record in records(body) {
            let Some((sid, new)) = RawPose::read_record(&self.schema, record)
            else {
                continue;
            };

            let track = self.tracks.entry(sid).or_insert_with(|| Track {
                raw: RawPose::default(),
                history: PoseHistory::new(history_cap),
            });
            track.raw.merge(&new);

            if let Some(pose) = track.raw.decode(&self.schema) {
                track.history.push(recv_t_s, pose);
            }
        }
    }

    pub fn sample(&self, sid: u32, at_t_s: f64) -> Option<PoseSample> {
        self.tracks.get(&sid)?.history.at(at_t_s)
    }

    pub fn forget(&mut self, sid: u32) { self.tracks.remove(&sid); }
}
