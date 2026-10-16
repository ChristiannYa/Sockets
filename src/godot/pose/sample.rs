//! [PoseSample], [POSE_STRIDE], blending, and angle lerping.
//! Adding a field starts here

use std::f32::consts::TAU;

/// Floats per pose in `UdpPoses.sample`: x, y, z, yaw, is_crouching.
/// Extends together with [PoseSample] and `to_array`.
pub const POSE_STRIDE: usize = 5;

#[derive(Clone, Copy)]
pub struct PoseSample {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub crouch: bool,
}

impl PoseSample {
    pub fn to_array(self) -> [f32; POSE_STRIDE] {
        [
            self.x,
            self.y,
            self.z,
            self.yaw,
            if self.crouch { 1.0 } else { 0.0 },
        ]
    }

    /// Slides from `older` to `newer`, `w` being the fraction of the way (0..1)
    pub fn blend(older: Self, newer: Self, w: f32) -> Self {
        PoseSample {
            x: lerp(older.x, newer.x, w),
            y: lerp(older.y, newer.y, w),
            z: lerp(older.z, newer.z, w),
            yaw: lerp_angle(older.yaw, newer.yaw, w),
            // A bool can't be blended, so hold the older value until the
            // newer sample's time arrives
            crouch: older.crouch,
        }
    }
}

fn lerp(from: f32, to: f32, w: f32) -> f32 { from + (to - from) * w }

/// Takes the shortest way around the circle
fn lerp_angle(from: f32, to: f32, w: f32) -> f32 {
    let diff = (to - from) % TAU;
    let dist = (2.0 * diff) % TAU - diff;
    from + dist * w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaw_takes_shortest_path() {
        // 350deg -> 10deg should pass through 0, not 180
        let a = 350f32.to_radians();
        let b = 10f32.to_radians();
        let mid = lerp_angle(a, b, 0.5);
        assert!(mid.cos() > 0.99, "mid = {mid}");
    }
}
