use super::FixedPoint;
use crate::pkt::FieldName;

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
