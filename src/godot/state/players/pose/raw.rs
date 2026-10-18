use super::PoseSample;
use crate::{bits::PacketSchema, pkt::FieldName, qua};

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
    /// Stores `val` if `name` is one of the pose fields, ignores it otherwise
    pub fn set(&mut self, name: &FieldName, val: u32) {
        match name {
            FieldName::LocationX => self.x = Some(val),
            FieldName::LocationY => self.y = Some(val),
            FieldName::LocationZ => self.z = Some(val),
            FieldName::Yaw => self.yaw = Some(val),
            FieldName::IsCrouching => self.is_crouching = Some(val),
            _ => {}
        }
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
