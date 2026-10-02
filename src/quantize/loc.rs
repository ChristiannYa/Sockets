use crate::{bits::PacketSchema, pkt::FieldName, quantize::fixed_pt::FixedPoint};

pub struct LocCodec {
    pub x: FixedPoint,
    pub z: FixedPoint,
}

pub fn loc_codec(schema: &PacketSchema) -> LocCodec {
    let fixed_pt =
        |field_name: &FieldName| FixedPoint::new(0.1, 128, schema.info_of(field_name).len);

    LocCodec {
        x: fixed_pt(&FieldName::LocationX),
        z: fixed_pt(&FieldName::LocationZ),
    }
}
