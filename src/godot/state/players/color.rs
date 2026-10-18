use crate::{bits::PacketSchema, pkt::FieldName, qua, util::Hsv};

/// Raw (still wire-encoded) color fields of one player
#[derive(Default)]
pub struct RawColor {
    h: Option<u32>,
    s: Option<u32>,
    v: Option<u32>,
}

impl RawColor {
    /// Stores `val` if `name` is one of the color fields, ignores it
    /// otherwise
    pub fn set(&mut self, name: &FieldName, val: u32) {
        match name {
            FieldName::ColorH => self.h = Some(val),
            FieldName::ColorS => self.s = Some(val),
            FieldName::ColorV => self.v = Some(val),
            _ => {}
        }
    }

    /// Overwrite with the fields that `new` carries (same rule as
    /// [`RawPose::merge`](super::pose::RawPose::merge))
    pub fn merge(&mut self, new: &RawColor) {
        self.h = new.h.or(self.h);
        self.s = new.s.or(self.s);
        self.v = new.v.or(self.v);
    }

    /// `None` until every colour field has been received at least once
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
