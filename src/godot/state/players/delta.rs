use super::raw::{RawColor, RawDev, RawPose, RawVel};
use crate::bits::{BitReader, PacketSchema};

#[derive(Default)]
pub struct PlayerDelta {
    pub dev: RawDev,
    pub pose: RawPose,
    pub vel: RawVel,
    pub color: RawColor,
}

impl PlayerDelta {
    /// Read a record in a single pass.
    /// Returns what the record carried, or [`None`] for a malformed record
    /// (!mask_fits())
    pub fn read(schema: &PacketSchema, record: &[u8]) -> Option<Self> {
        if !schema.mask_fits(record) {
            return None;
        }

        // Every set field must be read in schema order to keep the reader's
        // bit cursor in sync, even the ones no group wants
        let mut reader = BitReader::new(schema, record);
        let mut delta = PlayerDelta::default();

        for (name, _) in schema.fields.iter() {
            if !reader.isset(name) {
                continue;
            }

            let val = reader.read(name);
            delta.dev.set(name, val);
            delta.pose.set(name, val);
            delta.vel.set(name, val);
            delta.color.set(name, val)
        }

        Some(delta)
    }
}
