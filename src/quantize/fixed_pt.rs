use crate::bits::utils::mask;

pub struct FixedPoint {
    step: f32,
    offset: u32,
    bits_len: usize,
}

impl FixedPoint {
    pub const fn new(step: f32, offset: u32, bits_len: usize) -> Self {
        FixedPoint {
            step,
            offset,
            bits_len,
        }
    }

    pub fn encode(&self, val: f32) -> u32 {
        let step_ct = (val / self.step).round() as i32;
        let max = mask(&self.bits_len) as i32;
        (step_ct + self.offset as i32).clamp(0, max) as u32
    }

    pub fn decode(&self, val: u32) -> f32 {
        (val as i32 - self.offset as i32) as f32 * self.step
    }
}
