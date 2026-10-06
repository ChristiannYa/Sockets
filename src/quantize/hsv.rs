use crate::{
    bits::{PacketSchema, utils::mask},
    pkt::FieldName,
    quantize::fixed_pt::FixedPoint,
};

pub struct HsvCodec {
    pub h: FixedPoint,
    pub s: FixedPoint,
    pub v: FixedPoint,
}

pub fn hsv(schema: &PacketSchema) -> HsvCodec {
    let fixed_pt = |field_name: &FieldName| {
        let field = schema.info_of(field_name);
        let bits_max_val = mask(&field.len) as f32;
        FixedPoint::new(1.0 / bits_max_val, 0, field.len)
    };

    HsvCodec {
        h: fixed_pt(&FieldName::ColorH),
        s: fixed_pt(&FieldName::ColorS),
        v: fixed_pt(&FieldName::ColorV),
    }
}
