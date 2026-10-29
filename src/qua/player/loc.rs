use super::FixedPoint;
use crate::{pkt::FieldName, sim::consts::SPEED};

pub const VEL_Y: FixedPoint =
    FixedPoint::new(0.25, 128, FieldName::VelY.blen());

pub const VEL_HOR: FixedPoint =
    FixedPoint::new(SPEED / 31.0, 31, FieldName::VelX.blen());

pub struct LocCodec {
    pub x: FixedPoint,
    pub z: FixedPoint,
}

pub fn loc() -> LocCodec {
    let fixed_pt = |field_name: FieldName| {
        let bits_len = field_name.blen();
        let offset = 1u32 << (bits_len - 1); // half of 2^bits_len
        FixedPoint::new(0.1, offset, bits_len)
    };

    LocCodec {
        x: fixed_pt(FieldName::LocationX),
        z: fixed_pt(FieldName::LocationZ),
    }
}
