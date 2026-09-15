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

pub static FIELD_LENGTHS: &[(bits::FieldName, usize)] = &[
    (FieldName::SessionId, 6),
    (FieldName::Sequence, 8),
    (FieldName::LocationX, 8),
    (FieldName::LocationZ, 8),
    (FieldName::IsNewPlayer, 1),
    (FieldName::Health, 3),
    (FieldName::PlayerCount, 4),
    (FieldName::IsJumping, 1),
    (FieldName::IsCrouching, 1),
    (FieldName::IsFriendly, 1),
];

pub fn loc_codec(schema: &PacketSchema) -> LocCodec {
    let unit_m = 0.1;
    let offset = 128;

    LocCodec {
        x: FixedPoint {
            unit_m,
            offset,
            bits_len: schema.info_of(&FieldName::LocationX).len,
        },

        z: FixedPoint {
            unit_m,
            offset,
            bits_len: schema.info_of(&FieldName::LocationZ).len,
        },
    }
}
