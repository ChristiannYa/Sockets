pub const SIM_HZ: u32 = 60;

pub const TICK_DT_F32: f32 = 1.0 / SIM_HZ as f32;
pub const TICK_DT_F64: f64 = 1.0 / SIM_HZ as f64;

pub const SS_EVERY_TICKS: u8 = 1;
pub const SS_INTV_S: f64 = SS_EVERY_TICKS as f64 / SIM_HZ as f64;

pub const GRAVITY: f32 = 30.0;
pub const SPEED: f32 = 6.0;
pub const DECC: f32 = 20.0;
pub const JUMP_VEL: f32 = 14.0;
pub const CROUCH_SPEED: f32 = 6.0;
pub const CROUCH_HEIGHT: f32 = 1.0;

pub const HEIGHT: f32 = 2.0;
pub const RADIUS: f32 = 0.5;
pub const DIAM: f32 = 2.0 * RADIUS;

/// World y of the temporary flat floor.
pub const FLOOR_Y: f32 = 0.0;
