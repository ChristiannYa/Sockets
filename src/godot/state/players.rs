//! Every call across the Godot–Rust boundary (FFI) has a cost, so the work is
//! batched: one call per packet (`ingest_ss`) and one call per player per
//! frame (`sample`). Reading records, merging partial ones, decoding and
//! keeping the history all stay in Rust, between those two calls.

mod color;
mod delta;
mod pose;
mod records;

use crate::{
    bits::{DecodeType, PacketSchema},
    pkt::Seq,
};
use color::RawColor;
use delta::PlayerDelta;
use pose::PoseTrack;
pub use pose::{POSE_STRIDE, PoseSample};
use records::records;
use std::collections::HashMap;

use godot::prelude::{
    GodotClass, PackedByteArray, PackedFloat32Array, PackedInt32Array,
    godot_api,
};

struct PlayerState {
    pose: PoseTrack,
    color: RawColor,
    last_seq: Option<Seq>,
}

impl PlayerState {
    fn new(history_cap: usize) -> Self {
        PlayerState {
            pose: PoseTrack::new(history_cap),
            color: RawColor::default(),
            last_seq: None,
        }
    }

    /// Whether a record with this sequence number should be applied.
    ///
    /// Records without one are always applied. Otherwise the record must be
    /// newer than the last one applied, so late or repeated packets get
    /// dropped.
    fn apply_seq(&mut self, seq: Option<Seq>) -> bool {
        let Some(seq) = seq else {
            return true;
        };

        if self
            .last_seq
            .is_some_and(|last| !seq.is_newer_than(last))
        {
            return false;
        }

        self.last_seq = Some(seq);
        true
    }
}

/// State of every player, by session id
#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct PlayerStates {
    schema: PacketSchema,
    history_cap: usize,
    players: HashMap<u32, PlayerState>,
    my_sid: Option<u32>,
}

#[godot_api]
impl PlayerStates {
    #[constant]
    const POSE_STRIDE: i64 = POSE_STRIDE as i64;

    pub fn new(schema: PacketSchema, history_cap: usize) -> Self {
        PlayerStates {
            schema,
            history_cap,
            players: HashMap::new(),
            my_sid: None,
        }
    }

    /// This player's own session id, or -1 until the server has sent it
    #[func]
    pub fn my_sid(&self) -> i64 { self.my_sid.map_or(-1, i64::from) }

    /// Session ids of every player seen so far
    #[func]
    pub fn sids(&self) -> PackedInt32Array {
        let sids: Vec<i32> =
            self.players.keys().map(|&sid| sid as i32).collect();
        PackedInt32Array::from(sids.as_slice())
    }

    /// Reads a Data packet and updates the color of the players it carries.
    ///
    /// `buf` is `[PacketKind::Data, DecodeType]` followed by one record
    /// (single) or a records buffer (batch, layout in [`records::records`]).
    ///
    /// Only color is taken from these records. Their poses have no snapshot
    /// time, so only snapshots feed the pose history.
    ///
    /// Malformed records and records without a session id are skipped.
    #[func]
    pub fn ingest_data(&mut self, buf: PackedByteArray) {
        let bytes = buf.as_slice();
        let (Some(&decode_type), Some(body)) = (bytes.get(1), bytes.get(2..))
        else {
            return;
        };

        if decode_type == DecodeType::Single as u8 {
            self.apply_delta(body);
        } else if decode_type == DecodeType::Batch as u8 {
            for record in records(body) {
                self.apply_delta(record);
            }
        }
    }

    #[func]
    fn sample(&self, sid: i64, at_t_s: f64) -> PackedFloat32Array {
        pose_to_array(
            u32::try_from(sid).ok().and_then(|sid| {
                self.players.get(&sid)?.pose.history.at(at_t_s)
            }),
        )
    }

    #[func]
    fn latest(&self, sid: i64) -> PackedFloat32Array {
        pose_to_array(
            u32::try_from(sid).ok().and_then(|sid| {
                self.players.get(&sid)?.pose.history.latest()
            }),
        )
    }

    /// `[h, s, v]` of the player, or an empty array until every color field
    /// has been received
    #[func]
    fn color(&self, sid: i64) -> PackedFloat32Array {
        u32::try_from(sid)
            .ok()
            .and_then(|sid| {
                self.players.get(&sid)?.color.decode(&self.schema)
            })
            .map(|c| PackedFloat32Array::from(&[c.h, c.s, c.v][..]))
            .unwrap_or_default()
    }

    #[func]
    pub fn forget(&mut self, sid: i64) {
        if let Ok(sid) = u32::try_from(sid) {
            self.players.remove(&sid);
        }
    }
}

fn pose_to_array(pose: Option<PoseSample>) -> PackedFloat32Array {
    pose.map(|p| PackedFloat32Array::from(&p.to_array()[..]))
        .unwrap_or_default()
}

impl PlayerStates {
    fn apply_delta(&mut self, record: &[u8]) {
        let Some((sid, delta)) = PlayerDelta::read(&self.schema, record)
        else {
            return;
        };

        if delta.is_new {
            self.my_sid = Some(sid)
        }

        let player = self
            .players
            .entry(sid)
            .or_insert_with(|| PlayerState::new(self.history_cap));

        if !player.apply_seq(delta.seq) {
            return;
        }

        player.color.merge(&delta.color);
    }

    /// Updates each player's pose from a records buffer (layout in
    /// [`records::records`]).
    ///
    /// `t_s` is the time of the poses decoded from it. [`Self::sample`] looks
    /// poses up on that same clock.
    ///
    /// Malformed records and records without a session id are skipped.
    pub(super) fn apply_snapshot(&mut self, records_buf: &[u8], t_s: f64) {
        for record in records(records_buf) {
            let Some((sid, delta)) = PlayerDelta::read(&self.schema, record)
            else {
                continue;
            };

            let player = self
                .players
                .entry(sid)
                .or_insert_with(|| PlayerState::new(self.history_cap));

            player.pose.apply(&delta.pose, &self.schema, t_s);
            player.color.merge(&delta.color);
        }
    }
}
