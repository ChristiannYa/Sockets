use super::{
    history::PoseHistory, raw::RawPose, records::records, sample::PoseSample,
};
use crate::bits::PacketSchema;
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

    /// `body`'s the records buffer (`[record_count, (len, bytes)*]`) received
    /// at `recv_t_s`
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
