use super::FixedPoint;
use crate::{pkt::FieldName, sim::consts::SPEED};

pub const VEL_Y: FixedPoint =
    FixedPoint::new(0.25, 128, FieldName::VelY.blen());

pub const VEL_HOR: FixedPoint =
    FixedPoint::new(SPEED / 31.0, 31, FieldName::VelX.blen());

pub const LOC_HOR: FixedPoint = FieldName::LocationX.fpt();

pub const LOC_VER: FixedPoint =
    FixedPoint::new(0.05, 20, FieldName::LocationY.blen());

impl FieldName {
    const fn fpt(self) -> FixedPoint {
        let bits_len = self.blen();
        let offset = 1u32 << (bits_len - 1); // half of 2^bits_len
        FixedPoint::new(0.1, offset, bits_len)
    }
}
