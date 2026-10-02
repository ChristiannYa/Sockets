#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    pub mv_x: f32,
    pub mv_z: f32,
    pub yaw: f32,
    pub jump: bool,
    pub crouch: bool,
}

impl Input {
    pub fn new(mv_x: f32, mv_z: f32, yaw: f32, jump: bool, crouch: bool) -> Self {
        Input {
            mv_x,
            mv_z,
            yaw,
            jump,
            crouch,
        }
    }
}
