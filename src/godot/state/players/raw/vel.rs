use crate::{pkt::FieldName, qua};

#[derive(Default)]
pub struct RawVel {
    y: Option<u32>,
}

impl RawVel {
    pub fn set(&mut self, name: &FieldName, val: u32) {
        if let FieldName::VelY = name {
            self.y = Some(val);
        }
    }

    pub fn decode_y(&self) -> Option<f32> {
        Some(qua::player::VEL_Y.decode(self.y?))
    }
}
