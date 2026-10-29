use crate::{pkt::FieldName, qua, util::Hsv};

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

    pub fn decode(&self) -> Option<Hsv> {
        let hsv = qua::hsv();
        Some(Hsv {
            h: hsv.h.decode(self.h?),
            s: hsv.s.decode(self.s?),
            v: hsv.v.decode(self.v?),
        })
    }
}
