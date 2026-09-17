pub mod bits;
pub mod util;

#[cfg(feature = "godot")]
pub mod godot_bindings;

use crate::{
    bits::{FieldName, PacketSchema},
    util::fixed_pt::FixedPoint,
};

#[repr(u8)]
pub enum PacketKind {
    Single,
    Batch,
}

pub struct LocCodec {
    pub x: FixedPoint,
    pub z: FixedPoint,
}

pub struct HsvCodec {
    pub h: FixedPoint,
    pub s: FixedPoint,
    pub v: FixedPoint,
}

pub static FIELD_LENGTHS: &[(bits::FieldName, usize)] = &[
    (FieldName::DevSessionId, 6),
    (FieldName::DevSequence, 8),
    (FieldName::DevPing, 1),
    (FieldName::LocationX, 8),
    (FieldName::LocationZ, 8),
    (FieldName::ColorH, 8),
    (FieldName::ColorS, 8),
    (FieldName::ColorV, 8),
    (FieldName::DevIsNewPlayer, 1),
    (FieldName::Health, 3),
    (FieldName::PlayerCount, 4),
    (FieldName::IsJumping, 1),
    (FieldName::IsCrouching, 1),
    (FieldName::IsFriendly, 1),
];

pub fn loc_codec(schema: &PacketSchema) -> LocCodec {
    let fixed_pt = |field_name: &FieldName| FixedPoint {
        unit_m: 0.1,
        offset: 128,
        bits_len: schema.info_of(field_name).len,
    };

    LocCodec {
        x: fixed_pt(&FieldName::LocationX),
        z: fixed_pt(&FieldName::LocationZ),
    }
}

pub fn hsv_codec(schema: &PacketSchema) -> HsvCodec {
    let fixed_pt = |field_name: &FieldName| FixedPoint {
        unit_m: 0.1,
        offset: 128,
        bits_len: schema.info_of(field_name).len,
    };

    HsvCodec {
        h: fixed_pt(&FieldName::ColorH),
        s: fixed_pt(&FieldName::ColorS),
        v: fixed_pt(&FieldName::ColorV),
    }
}
