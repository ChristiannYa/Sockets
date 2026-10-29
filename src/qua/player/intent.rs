use crate::{pkt::FieldName, qua::fixed_pt::FixedPoint};
use std::f32::consts::TAU;

pub const MV: FixedPoint =
    FixedPoint::new(1.0 / 31.0, 31, FieldName::IntentMvX.blen());

pub const YAW: FixedPoint =
    FixedPoint::new(TAU / 1024.0, 512, FieldName::IntentYaw.blen());

pub const Y: FixedPoint =
    FixedPoint::new(0.05, 20, FieldName::LocationY.blen());
