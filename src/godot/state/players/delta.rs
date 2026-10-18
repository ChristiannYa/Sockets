use super::pose::RawPose;
use crate::{
    bits::{BitReader, PacketSchema},
    pkt::FieldName,
};

/// What records carry
pub struct PlayerDelta {
    pub pose: RawPose,
}

impl PlayerDelta {
    /// Read a record in a single pass.
    /// Returns the session id and what the record carried, or [`None`] for a
    /// malformed record or one without an id.
    pub fn read(schema: &PacketSchema, record: &[u8]) -> Option<(u32, Self)> {
        if !schema.mask_fits(record) {
            return None;
        }

        // Every set field must be read in schema order to keep the reader's
        // bit cursor in sync, even the ones no group wants
        let mut reader = BitReader::new(schema, record);
        let mut sid = None;
        let mut delta = PlayerDelta { pose: RawPose::default() };

        for (name, _) in schema.fields.iter() {
            if !reader.isset(name) {
                continue;
            }
            let val = reader.read(name);
            match name {
                FieldName::DevSessionId => sid = Some(val),
                _ => delta.pose.set(name, val),
            }
        }

        Some((sid?, delta))
    }
}
