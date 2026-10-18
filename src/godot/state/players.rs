//! Everything the game knows about each player, kept in Rust.
//!
//! Every call across the Godot–Rust boundary (FFI) has a cost, so the work is
//! batched: one call per packet (`ingest_ss`) and one call per player per
//! frame (`sample`). Reading records, merging partial ones, decoding and
//! keeping the history all stay in Rust, between those two calls.

mod color;
mod delta;
mod pose;
mod records;

use crate::bits::{DecodeType, PacketSchema};
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

/// `[PacketKind::Snapshot, tick_lo, tick_hi]`, followed by the records
const SNAPSHOT_HEADER_LEN: usize = 3;

/// Everything known about one player. Color joins `pose` here later.
struct PlayerState {
    pose: PoseTrack,
    color: RawColor,
}

impl PlayerState {
    fn new(history_cap: usize) -> Self {
        PlayerState {
            pose: PoseTrack::new(history_cap),
            color: RawColor::default(),
        }
    }
}

/// State of every player, by session id
#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct PlayerStates {
    schema: PacketSchema,
    history_cap: usize,
    players: HashMap<u32, PlayerState>,
}

#[godot_api]
impl PlayerStates {
    #[constant]
    const POSE_STRIDE: i64 = POSE_STRIDE as i64;

    pub fn new(schema: PacketSchema, history_cap: usize) -> Self {
        PlayerStates { schema, history_cap, players: HashMap::new() }
    }

    /// Session ids of every player seen so far
    #[func]
    pub fn sids(&self) -> PackedInt32Array {
        let sids: Vec<i32> =
            self.players.keys().map(|&sid| sid as i32).collect();
        PackedInt32Array::from(sids.as_slice())
    }

    /// Reads a snapshot packet and updates each player's state.
    ///
    /// `buf` is the whole packet: `[PacketKind::Snapshot, tick_lo, tick_hi]`
    /// followed by a records buffer (layout in [`records::records`]).
    /// `recv_t_s` is the time of the poses decoded from it. [`Self::sample`]
    /// looks poses up on that same clock.
    ///
    /// A packet too short to have a header is ignored, and so are malformed
    /// records and records without a session id.
    #[func]
    pub fn ingest_ss(&mut self, buf: PackedByteArray, recv_t_s: f64) {
        if let Some(body) = buf.as_slice().get(SNAPSHOT_HEADER_LEN..) {
            let history_cap = self.history_cap;

            for record in records(body) {
                let Some((sid, delta)) =
                    PlayerDelta::read(&self.schema, record)
                else {
                    continue;
                };

                let player = self
                    .players
                    .entry(sid)
                    .or_insert_with(|| PlayerState::new(history_cap));

                player.pose.apply(&delta.pose, &self.schema, recv_t_s);
                player.color.merge(&delta.color);
            }
        }
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
            self.merge_color(body);
        } else if decode_type == DecodeType::Batch as u8 {
            for record in records(body) {
                self.merge_color(record);
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
    fn merge_color(&mut self, record: &[u8]) {
        let Some((sid, delta)) = PlayerDelta::read(&self.schema, record)
        else {
            return;
        };
        let history_cap = self.history_cap;
        let player = self
            .players
            .entry(sid)
            .or_insert_with(|| PlayerState::new(history_cap));
        player.color.merge(&delta.color);
    }
}
