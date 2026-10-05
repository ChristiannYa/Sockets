use crate::sim::{
    consts::{
        CROUCH_HEIGHT, CROUCH_SPEED, DECC, FLOOR_Y, GRAVITY, HEIGHT, JUMP_VEL, SPEED, TICK_DT,
    },
    intent::PlayerIntentSim,
    state::PlayerState,
};

pub struct Step<'a> {
    p: &'a mut PlayerState,
    sim_intent: &'a PlayerIntentSim,
}

impl<'a> Step<'a> {
    pub fn new(p: &'a mut PlayerState, sim_intent: &'a PlayerIntentSim) -> Self {
        Step { p, sim_intent }
    }

    pub fn run(&mut self) {
        self.apply_gravity();

        self.mv_x_vel();
        self.mv_y_jump();
        self.mv_y_crouch();

        self.integrate();
        self.temp_floor();
    }

    fn apply_gravity(&mut self) {
        self.p.vel[1] -= GRAVITY * TICK_DT;
    }

    fn mv_x_vel(&mut self) {
        let (x, z) = wish_dir(
            self.sim_intent.held.mv_x,
            self.sim_intent.held.mv_z,
            self.sim_intent.held.yaw,
        );
        if x.hypot(z) > 0.01 {
            self.p.vel[0] = x * SPEED;
            self.p.vel[2] = z * SPEED;
        } else {
            self.p.vel[0] = move_toward(self.p.vel[0], 0.0, DECC * TICK_DT);
            self.p.vel[2] = move_toward(self.p.vel[2], 0.0, DECC * TICK_DT);
        }
    }

    fn mv_y_jump(&mut self) {
        if self.sim_intent.edges.jump && self.p.is_on_floor {
            self.p.vel[1] = JUMP_VEL
        }
    }

    fn mv_y_crouch(&mut self) {
        if self.p.is_on_floor {
            let dir = if self.sim_intent.held.crouch {
                -1.0
            } else {
                1.0
            };
            let h = self.p.height + dir * CROUCH_SPEED * TICK_DT;
            self.p.height = h.clamp(CROUCH_HEIGHT, HEIGHT);
        }
    }

    fn integrate(&mut self) {
        self.p.pos[0] += self.p.vel[0] * TICK_DT;
        self.p.pos[1] += self.p.vel[1] * TICK_DT;
        self.p.pos[2] += self.p.vel[2] * TICK_DT;
    }

    fn temp_floor(&mut self) {
        if self.p.pos[1] <= FLOOR_Y {
            self.p.pos[1] = FLOOR_Y;
            self.p.vel[1] = 0.0;
            self.p.is_on_floor = true;
        } else {
            self.p.is_on_floor = false;
        }
    }
}

fn wish_dir(x: f32, z: f32, yaw: f32) -> (f32, f32) {
    let mag = x.hypot(z);

    let (mv_x, mv_z) = if mag > 1.0 {
        (x / mag, z / mag)
    } else {
        (x, z)
    };

    let (yaw_sin, yaw_cos) = yaw.sin_cos();
    (
        yaw_cos * mv_x + yaw_sin * mv_z,
        -yaw_sin * mv_x + yaw_cos * mv_z,
    )
}

fn move_toward(from: f32, to: f32, delta: f32) -> f32 {
    if (to - from).abs() <= delta {
        to
    } else {
        from + (to - from).signum() * delta
    }
}
