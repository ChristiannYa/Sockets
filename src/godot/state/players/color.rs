use crate::{bits::PacketSchema, pkt::FieldName, qua, util::Hsv};

#[derive(Default)]
pub struct RawColor {
    h: Option<u32>,
    s: Option<u32>,
    v: Option<u32>,
}

impl RawColor {
    pub fn set(&mut self, name: &FieldName, val: u32) {
        match name {
            FieldName::ColorH => self.h = Some(val),
            FieldName::ColorS => self.s = Some(val),
            FieldName::ColorV => self.v = Some(val),
            _ => {}
        }
    }

    pub fn merge(&mut self, new: &RawColor) {
        self.h = new.h.or(self.h);
        self.s = new.s.or(self.s);
        self.v = new.v.or(self.v);
    }

    pub fn decode(&self, schema: &PacketSchema) -> Option<Hsv> {
        let hsv = qua::hsv(schema);
        Some(Hsv {
            h: hsv.h.decode(self.h?),
            s: hsv.s.decode(self.s?),
            v: hsv.v.decode(self.v?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkt::FIELD_LENGTHS;

    #[test]
    fn decodes_only_when_every_field_is_known() {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        let mut color = RawColor::default();

        color.set(&FieldName::ColorH, 1);
        color.set(&FieldName::ColorS, 1);
        assert!(color.decode(&schema).is_none());

        color.set(&FieldName::ColorV, 1);
        assert!(color.decode(&schema).is_some());
    }

    #[test]
    fn merge_keeps_what_the_new_one_lacks() {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        let mut stored = RawColor::default();
        stored.set(&FieldName::ColorH, 1);
        stored.set(&FieldName::ColorS, 1);
        stored.set(&FieldName::ColorV, 1);

        let mut new = RawColor::default();
        new.set(&FieldName::ColorH, 2);
        stored.merge(&new);

        assert!(stored.decode(&schema).is_some());
    }
}
