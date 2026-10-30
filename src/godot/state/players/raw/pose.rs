use super::super::pose::PoseSample;
use crate::{pkt::FieldName, qua};

#[derive(Default)]
pub struct RawPose {
    x: Option<u32>,
    y: Option<u32>,
    z: Option<u32>,
    yaw: Option<u32>,
    is_crouching: Option<u32>,
}

impl RawPose {
    pub fn set(&mut self, name: &FieldName, val: u32) {
        match name {
            FieldName::LocationX => self.x = Some(val),
            FieldName::LocationY => self.y = Some(val),
            FieldName::LocationZ => self.z = Some(val),
            FieldName::Yaw => self.yaw = Some(val),
            FieldName::IsCrouching => self.is_crouching = Some(val),
            _ => {}
        }
    }

    pub fn merge(&mut self, new: &RawPose) {
        self.x = new.x.or(self.x);
        self.y = new.y.or(self.y);
        self.z = new.z.or(self.z);
        self.yaw = new.yaw.or(self.yaw);
        self.is_crouching = new.is_crouching.or(self.is_crouching);
    }

    pub fn decode(&self) -> Option<PoseSample> {
        let loc_hor_qua = qua::player::LOC_HOR;
        Some(PoseSample {
            x: loc_hor_qua.decode(self.x?),
            y: qua::player::LOC_VER.decode(self.y?),
            z: loc_hor_qua.decode(self.z?),
            yaw: qua::player::intent::YAW.decode(self.yaw?),
            is_crouching: self.is_crouching? == 1,
        })
    }
}
