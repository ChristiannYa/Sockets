#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    pub move_x: f32,
    pub move_z: f32,
    pub yaw: f32,
    pub jump: bool,
    pub crouch: bool,
}
