//! Smooth movement of remote players.
//!
//! The server sends every player's pose in snapshots 60 times a second.
//! Drawing each pose as it arrives would look jittery, so poses are kept with
//! their time and the game asks for the pose at a moment slightly in the past,
//! blended between the two snapshots around that moment.
//!
//! Every call across the Godot–Rust boundary (FFI) has a cost, so the work is
//! batched: one call per packet (`ingest`) and one call per player per frame
//! (`sample`). Reading records, merging partial ones, decoding and keeping the
//! history all stay in Rust, between those two calls.

mod sample;
mod tracker;

use godot::prelude::{
    Gd, GodotClass, PackedByteArray, PackedFloat32Array, godot_api,
};
use sample::{POSE_STRIDE, PoseSample};
use tracker::PoseTracker;

use crate::{bits::PacketSchema, pkt::FIELD_LENGTHS};

/// `[PacketKind::Snapshot, tick_lo, tick_hi]`, followed by the records
const SNAPSHOT_HEADER_LEN: usize = 3;

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpPoses {
    tracker: PoseTracker,
}

#[godot_api]
impl UdpPoses {
    #[constant]
    const POSE_STRIDE: i64 = POSE_STRIDE as i64;

    #[func]
    fn create(history_cap: i64) -> Gd<Self> {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        let cap = history_cap.max(2) as usize;
        Gd::from_init_fn(|_| Self { tracker: PoseTracker::new(schema, cap) })
    }

    #[func]
    fn ingest_snapshot(&mut self, buf: PackedByteArray, recv_t_s: f64) {
        if let Some(body) = buf.as_slice().get(SNAPSHOT_HEADER_LEN..) {
            self.tracker.ingest(body, recv_t_s);
        }
    }

    #[func]
    fn sample(&self, sid: i64, at_t_s: f64) -> PackedFloat32Array {
        pose_to_array(
            u32::try_from(sid)
                .ok()
                .and_then(|sid| self.tracker.sample(sid, at_t_s)),
        )
    }

    #[func]
    fn latest(&self, sid: i64) -> PackedFloat32Array {
        pose_to_array(
            u32::try_from(sid)
                .ok()
                .and_then(|sid| self.tracker.latest(sid)),
        )
    }

    #[func]
    fn forget(&mut self, sid: i64) {
        if let Ok(sid) = u32::try_from(sid) {
            self.tracker.forget(sid);
        }
    }
}

fn pose_to_array(pose: Option<PoseSample>) -> PackedFloat32Array {
    pose.map(|p| PackedFloat32Array::from(&p.to_array()[..]))
        .unwrap_or_default()
}
