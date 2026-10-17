mod sample;
mod tracker;

use godot::prelude::{
    Gd, GodotClass, PackedByteArray, PackedFloat32Array, godot_api,
};
use sample::POSE_STRIDE;
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

    /// `[x, y, z, yaw, ...]`, or empty when there is no data yet
    #[func]
    fn sample(&self, sid: i64, at_t_s: f64) -> PackedFloat32Array {
        u32::try_from(sid)
            .ok()
            .and_then(|sid| self.tracker.sample(sid, at_t_s))
            .map(|pose| PackedFloat32Array::from(&pose.to_array()[..]))
            .unwrap_or_default()
    }

    #[func]
    fn forget(&mut self, sid: i64) {
        if let Ok(sid) = u32::try_from(sid) {
            self.tracker.forget(sid);
        }
    }
}
