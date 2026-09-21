use crate::{bits::schema::PacketSchema, fields::names::FieldName, quantize::fixed_pt::FixedPoint};

pub struct LocCodec {
    pub x: FixedPoint,
    pub z: FixedPoint,
}

pub fn loc_codec(schema: &PacketSchema) -> LocCodec {
    let fixed_pt = |field_name: &FieldName| FixedPoint {
        step: 0.1,
        offset: 128,
        bits_len: schema.info_of(field_name).len,
    };

    LocCodec {
        x: fixed_pt(&FieldName::LocationX),
        z: fixed_pt(&FieldName::LocationZ),
    }
}
