use crate::quantize::fixed_pt::FixedPoint;

pub const MV: FixedPoint = FixedPoint::new(1.0 / 31.0, 31, 6);

pub const YAW: FixedPoint = FixedPoint::new(std::f32::consts::TAU / 1024.0, 512, 10);
