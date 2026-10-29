use crate::{pkt::FieldName, qua};

#[derive(Default)]
pub struct RawVel {
    x: Option<u32>,
    y: Option<u32>,
    z: Option<u32>,
}

impl RawVel {
    pub fn set(&mut self, name: &FieldName, val: u32) {
        match name {
            FieldName::VelX => self.x = Some(val),
            FieldName::VelY => self.y = Some(val),
            FieldName::VelZ => self.z = Some(val),
            _ => {}
        }
    }

    pub fn decode(&self) -> Option<(f32, f32, f32)> {
        Some((
            qua::player::VEL_HOR.decode(self.x?),
            qua::player::VEL_Y.decode(self.y?),
            qua::player::VEL_HOR.decode(self.z?),
        ))
    }
}
