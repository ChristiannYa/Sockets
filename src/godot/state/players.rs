//! Every call across the Godot–Rust boundary (FFI) has a cost, so the work is
//! batched: one call per packet (`ingest_ss`) and one call per player per
//! frame (`sample`). Reading records, merging partial ones, decoding and
//! keeping the history all stay in Rust, between those two calls.

mod debug;
mod delta;
mod pose;
mod raw;
mod records;

use super::GAP_WARN_S;
use crate::{
    bits::{DecodeType, PacketSchema},
    pkt::Seq,
};
use debug::PlayersStateDbg;
use delta::PlayerDelta;
use pose::PoseTrack;
pub use pose::{POSE_STRIDE, PoseSample};
use raw::RawColor;
use std::collections::HashMap;

use godot::prelude::{
    GodotClass, PackedByteArray, PackedFloat32Array, PackedInt32Array,
    godot_api,
};

struct PlayerDev {
    sid: u32,
    seq: Option<Seq>,
    simed_seq: Option<Seq>,
    im_new: bool,
}

/// What the newest snapshot said about the local player, as one consistent
/// union for the predictor to reconcile against. Handed over once, see
/// [`PlayerStates::take_my_server_state`]
pub struct MyServerState {
    pub simed_seq: Seq,
    pub pose: PoseSample,
    pub vel_y: f32,
}

struct PlayerState {
    pose: PoseTrack,
    color: RawColor,
    last_seq: Option<Seq>,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            pose: PoseTrack::new(),
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
    players: HashMap<u32, PlayerState>,
    my_sid: Option<u32>,
    my_server_state: Option<MyServerState>,
    dbg: PlayersStateDbg,
}

#[godot_api]
impl PlayerStates {
    #[constant]
    const POSE_STRIDE: i64 = POSE_STRIDE as i64;

    pub fn new(schema: PacketSchema) -> Self {
        PlayerStates {
            schema,
            players: HashMap::new(),
            my_sid: None,
            my_server_state: None,
            dbg: PlayersStateDbg::default(),
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
    /// (single) or a records buffer (batch, layout in [`records::split`]).
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
            for record in records::split(body) {
                self.apply_delta(record);
            }
        }
    }

    #[func]
    fn sample(&mut self, sid: i64, t_s: f64) -> PackedFloat32Array {
        self.dbg.frzw.on_sample(t_s);
        pose_to_array(
            u32::try_from(sid)
                .ok()
                .and_then(|sid| self.players.get(&sid)?.pose.at(t_s)),
        )
    }

    #[func]
    fn latest(&mut self, sid: i64) -> PackedFloat32Array {
        self.dbg.frzw.on_latest();
        pose_to_array(
            u32::try_from(sid)
                .ok()
                .and_then(|sid| self.players.get(&sid)?.pose.latest()),
        )
    }

    /// `[h, s, v]` of the player, or an empty array until every color field
    /// has been received
    #[func]
    fn color(&self, sid: i64) -> PackedFloat32Array {
        u32::try_from(sid)
            .ok()
            .and_then(|sid| self.players.get(&sid)?.color.decode())
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
        let Some(delta) = PlayerDelta::read(&self.schema, record) else {
            return;
        };

        let Some(dev) = delta.dev.decode() else {
            return;
        };

        if dev.im_new {
            self.my_sid = Some(dev.sid)
        }

        let player = self
            .players
            .entry(dev.sid)
            .or_insert_with(PlayerState::new);

        if !player.apply_seq(dev.seq) {
            return;
        }

        player.color.merge(&delta.color);
    }

    /// Updates each player's pose from a records buffer (layout in
    /// [`records::split`]).
    ///
    /// `t_s` is the time of the poses decoded from it. [`Self::sample`] looks
    /// poses up on that same clock.
    ///
    /// Malformed records and records without a session id are skipped.
    pub(super) fn apply_ss(&mut self, records_buf: &[u8], t_s: f64) {
        self.dbg.frzw.on_ss(t_s);

        for record in records::split(records_buf) {
            let Some(delta) = PlayerDelta::read(&self.schema, record) else {
                continue;
            };

            let Some(dev) = delta.dev.decode() else {
                continue;
            };

            let player = self
                .players
                .entry(dev.sid)
                .or_insert_with(PlayerState::new);

            let pose = player.pose.apply(&delta.pose, t_s);

            if Some(dev.sid) == self.my_sid
                && let (Some(pose), Some(simed_seq), Some(vel_y)) =
                    (pose, dev.simed_seq, delta.vel.decode_y())
            {
                self.my_server_state =
                    Some(MyServerState { simed_seq, vel_y, pose })
            }
        }
    }

    /// Hand over the newest server state of the local player, and clears it.
    ///
    /// The predictor asks every client tick, but a snapshot arrives only now
    /// and then. Clearing on read makes each snapshot reconcile exactly once:
    /// [`None`] means "nothing new, keep predicting", so a stale snapshot is
    /// never rebased onto again.
    ///
    /// If several snapshots arrive between two asks, only the newest is kept
    pub(super) fn take_my_server_state(&mut self) -> Option<MyServerState> {
        self.my_server_state.take()
    }
}
