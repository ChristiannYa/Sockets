//! Every call across the Godot–Rust boundary (FFI) has a cost, so the work is
//! batched: one call per packet (`ingest_ss`) and one call per player per
//! frame (`sample`). Reading records, merging partial ones, decoding and
//! keeping the history all stay in Rust, between those two calls.

mod delta;
mod pose;
mod raw;
mod records;

use super::GAP_WARN_S;
use crate::{
    bits::{DecodeType, PacketSchema},
    logt,
    pkt::Seq,
};
use delta::PlayerDelta;
use pose::PoseTrack;
pub use pose::{POSE_STRIDE, PoseSample};
use raw::RawColor;
use std::{collections::HashMap, time::Instant};

use godot::prelude::{
    GodotClass, PackedByteArray, PackedFloat32Array, PackedInt32Array,
    godot_api,
};

/// How far the render time must be past the newest snapshot to count as a
/// freeze (~1 server tick)
const FREEZE_MIN_S: f64 = 0.020;

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

    /// Time of the newest snapshot applied, on the local clock
    newest_ss_s: Option<f64>,

    /// Render time when the current freeze began. [`None`] while smooth.
    frozen_since_s: Option<f64>,

    /// When the newest snapshot was applied
    last_ss_at: Option<Instant>,

    /// When [`Self::latest`] last ran, which is about once per frame
    last_frame_at: Option<Instant>,

    /// A STALL line was already logged for the current silence
    stall_logged: bool,
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
            newest_ss_s: None,
            frozen_since_s: None,
            last_ss_at: None,
            last_frame_at: None,
            stall_logged: false,
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
        self.watch_freeze(t_s);
        pose_to_array(
            u32::try_from(sid)
                .ok()
                .and_then(|sid| self.players.get(&sid)?.pose.at(t_s)),
        )
    }

    #[func]
    fn latest(&mut self, sid: i64) -> PackedFloat32Array {
        self.watch_stall();
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
    /// [`records::split`]).
    ///
    /// `t_s` is the time of the poses decoded from it. [`Self::sample`] looks
    /// poses up on that same clock.
    ///
    /// Malformed records and records without a session id are skipped.
    pub(super) fn apply_snapshot(&mut self, records_buf: &[u8], t_s: f64) {
        self.newest_ss_s = Some(t_s);
        self.last_ss_at = Some(Instant::now());
        self.stall_logged = false;

        for record in records::split(records_buf) {
            let Some((sid, delta)) = PlayerDelta::read(&self.schema, record)
            else {
                continue;
            };

            let player = self
                .players
                .entry(sid)
                .or_insert_with(|| PlayerState::new(self.history_cap));

            player.pose.apply(&delta.pose, &self.schema, t_s);
        }
    }

    /// Logs once when the render time passes the newest snapshot (no more
    /// data to play) and once when it recovers.
    fn watch_freeze(&mut self, t_s: f64) {
        let Some(newest_ss_s) = self.newest_ss_s else {
            return;
        };
        let past_s = t_s - newest_ss_s;

        match (self.frozen_since_s, past_s > FREEZE_MIN_S) {
            (None, true) => {
                self.frozen_since_s = Some(t_s);
                logt!(
                    "FREEZE: render {:.0}ms past the newest",
                    past_s * 1000.0
                );
            }
            (Some(since), false) => {
                self.frozen_since_s = None;
                logt!("UNFROZEN: after {:.0}ms", (t_s - since) * 1000.0);
            }
            _ => {}
        }
    }

    /// Logs once, while it is happening, when no snapshot has arrived for
    /// longer than [`GAP_WARN_S`]. [`Self::latest`] runs every frame for the
    /// local player, so this fires mid-freeze instead of after it.
    fn watch_stall(&mut self) {
        let now = Instant::now();
        let frame_gap_s = self
            .last_frame_at
            .replace(now)
            .map(|prev| now.duration_since(prev).as_secs_f64());

        let Some(ss_at) = self.last_ss_at else {
            return;
        };
        let since_ss_s = now.duration_since(ss_at).as_secs_f64();

        if since_ss_s > GAP_WARN_S && !self.stall_logged {
            self.stall_logged = true;
            logt!(
                "STALL: no snapshot for {:.0}ms and counting (last frame took {:.0}ms)",
                since_ss_s * 1000.0,
                frame_gap_s.unwrap_or(0.0) * 1000.0
            );
        }
    }
}
