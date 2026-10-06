use crate::{
    bits::PacketSchema, pkt::FieldName, quantize::fixed_pt::FixedPoint,
};

pub struct LocCodec {
    pub x: FixedPoint,
    pub z: FixedPoint,
}

pub fn loc(schema: &PacketSchema) -> LocCodec {
    let fixed_pt = |field_name: &FieldName| {
        let bits_len = schema.info_of(field_name).len;
        let offset = 1u32 << (bits_len - 1); // half of 2^bits_len
        FixedPoint::new(0.1, offset, bits_len)
    };

    LocCodec {
        x: fixed_pt(&FieldName::LocationX),
        z: fixed_pt(&FieldName::LocationZ),
    }
}
