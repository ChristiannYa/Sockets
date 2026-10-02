#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    pub mv_x: f32,
    pub mv_z: f32,
    pub yaw: f32,
    pub jump: bool,
    pub crouch: bool,
}
