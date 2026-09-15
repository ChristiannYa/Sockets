use crate::util::mask;

pub struct FixedPoint {
    pub unit_m: f32,
    pub offset: u32,
    pub bits_len: usize,
}

impl FixedPoint {
    pub fn encode(&self, val: f32) -> u32 {
        let unit_m_ct = (val / self.unit_m).round() as i32;
        let max = mask(&self.bits_len) as i32;
        (unit_m_ct + self.offset as i32).clamp(0, max) as u32
    }

    pub fn decode(&self, val: u32) -> f32 {
        (val as i32 - self.offset as i32) as f32 * self.unit_m
    }
}
