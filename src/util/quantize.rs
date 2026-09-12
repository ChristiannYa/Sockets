pub struct FixedPoint {
    pub step: f32,
    pub offset: i32,
    pub max: u32,
}

impl FixedPoint {
    pub fn encode(&self, val: f32) -> u32 {
        ((val / self.step).round() as i32 + self.offset).clamp(0, self.max as i32) as u32
    }

    pub fn decode(&self, val: u32) -> f32 {
        (val as i32 - self.offset) as f32 * self.step
    }
}
