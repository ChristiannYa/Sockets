use super::sample::PoseSample;
use crate::{
    bits::{BitReader, PacketSchema},
    pkt::FieldName,
    qua,
};

/// Raw (still wire-encoded) pose fields of one player.
#[derive(Default)]
pub struct RawPose {
    x: Option<u32>,
    y: Option<u32>,
    z: Option<u32>,
    yaw: Option<u32>,
    is_crouching: Option<u32>,
}

impl RawPose {
    /// Read a record.
    /// Returns the session id and the pose fields the record carried, or
    /// `None` for a malformed record or one without an id.
    pub fn read_record(
        schema: &PacketSchema,
        record: &[u8],
    ) -> Option<(u32, Self)> {
        if !schema.mask_fits(record) {
            return None;
        }

        // Every set field must be read in schema order to keep the reader's
        // bit cursor in sync, even the ones ignored here
        let mut reader = BitReader::new(schema, record);
        let mut sid = None;
        let mut raw = RawPose::default();

        for (name, _) in schema.fields.iter() {
            if !reader.isset(name) {
                continue;
            }
            let val = reader.read(name);
            match name {
                FieldName::DevSessionId => sid = Some(val),
                FieldName::LocationX => raw.x = Some(val),
                FieldName::LocationY => raw.y = Some(val),
                FieldName::LocationZ => raw.z = Some(val),
                FieldName::Yaw => raw.yaw = Some(val),
                FieldName::IsCrouching => raw.is_crouching = Some(val),
                _ => {}
            }
        }

        Some((sid?, raw))
    }

    /// Overwrite with fields that `new` carries.
    /// `new`'s [Some] values replace what is stored and its [None] values are
    /// ignored (update only what's present).
    ///
    /// ```text
    /// stored: x=10, y=10, z=5
    /// new:    x=12, y=None, z=None
    /// result: x=12, y=10, z=5
    /// ```
    pub fn merge(&mut self, new: &RawPose) {
        self.x = new.x.or(self.x);
        self.y = new.y.or(self.y);
        self.z = new.z.or(self.z);
        self.yaw = new.yaw.or(self.yaw);
        self.is_crouching = new.is_crouching.or(self.is_crouching);
    }

    /// `None` until every pose field has been received at least once
    pub fn decode(&self, schema: &PacketSchema) -> Option<PoseSample> {
        let loc = qua::loc(schema);
        Some(PoseSample {
            x: loc.x.decode(self.x?),
            y: qua::intent::Y.decode(self.y?),
            z: loc.z.decode(self.z?),
            yaw: qua::intent::YAW.decode(self.yaw?),
            is_crouching: self.is_crouching? == 1,
        })
    }
}
