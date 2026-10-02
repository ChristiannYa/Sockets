use crate::sim::{
    consts::{
        CROUCH_HEIGHT, CROUCH_SPEED, DECC, FLOOR_Y, GRAVITY, HEIGHT, JUMP_VEL, SPEED, TICK_DT,
    },
    input::Input,
    state::PlayerState,
};

pub struct Step<'a> {
    p: &'a mut PlayerState,
    inp: &'a Input,
}

impl<'a> Step<'a> {
    pub fn new(p: &'a mut PlayerState, inp: &'a Input) -> Self {
        Step { p, inp }
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
        let (x, z) = wish_dir(self.inp.mv_x, self.inp.mv_z, self.inp.yaw);
        if x.hypot(z) > 0.01 {
            self.p.vel[0] = x * SPEED;
            self.p.vel[2] = z * SPEED;
        } else {
            self.p.vel[0] = move_toward(self.p.vel[0], 0.0, DECC * TICK_DT);
            self.p.vel[2] = move_toward(self.p.vel[2], 0.0, DECC * TICK_DT);
        }
    }

    fn mv_y_jump(&mut self) {
        if self.inp.jump && self.p.is_on_floor {
            self.p.vel[1] = JUMP_VEL
        }
    }

    fn mv_y_crouch(&mut self) {
        if self.p.is_on_floor {
            let dir = if self.inp.crouch { -1.0 } else { 1.0 };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn gravity_spawned_stays_put() {
        let mut p = PlayerState::spawn(3.0, -2.0);
        let inp = Input::default();

        for _ in 0..60 {
            Step::new(&mut p, &inp).run();
        }

        assert_eq!(p.pos, [3.0, FLOOR_Y, -2.0]);
        assert!(p.is_on_floor);
    }

    #[test]
    fn gravity_lands_on_time() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        p.pos[1] = FLOOR_Y + 5.0;
        p.is_on_floor = false;
        let inp = Input::default();

        let mut ticks = 0;
        while !p.is_on_floor && ticks < 1000 {
            Step::new(&mut p, &inp).run();
            ticks += 1;
        }

        // Free fall from 5m at g=30: sqrt(2*5/30) = 0.577s. ~35 ticks
        assert!((34..=37).contains(&ticks), "landed after {ticks} ticks");
        assert_eq!(p.pos[1], FLOOR_Y)
    }

    #[test]
    fn mv_x_fwd_inp_reaches_full_speed_immediately() {
        let mut p = PlayerState::spawn(0.0, 0.0);

        // In Godot, "forward" is -z
        let inp = Input {
            mv_z: -1.0,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();

        assert!(approx(p.vel[2], -6.0));
        assert!(approx(p.pos[2], -6.0 * TICK_DT));
    }

    #[test]
    fn mv_x_yaw_rotates_movement() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            mv_z: -1.0,
            yaw: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();

        // Facing +90deg around y, "forward" points along -x
        assert!(approx(p.vel[0], -6.0));
        assert!(approx(p.vel[2], 0.0));
    }

    #[test]
    fn mv_x_diagonal_input_is_clamped() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            mv_x: 1.0,
            mv_z: 1.0,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();

        let speed = (p.vel[0].powi(2) + p.vel[2].powi(2)).sqrt();
        assert!(approx(speed, 6.0), "speed was {speed}");
    }

    #[test]
    fn mv_x_inp_release_stops_in_about_a_third_of_a_second() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            mv_z: -1.0,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();

        let mut ticks = 0;
        while p.vel[2] != 0.0 && ticks < 100 {
            Step::new(&mut p, &Input::default()).run();
            ticks += 1;
        }

        // 6 / 20 = 0.3s = 18 ticks
        assert!((17..=19).contains(&ticks), "stopped after {ticks} ticks");
    }

    #[test]
    // "Apex": Highest point of a jump
    fn mv_y_jump_airtime_and_apex() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            jump: true,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();
        assert!(!p.is_on_floor);

        let mut ticks = 1;
        let mut apex = p.pos[1];
        while !p.is_on_floor && ticks < 200 {
            Step::new(&mut p, &Input::default()).run();
            apex = apex.max(p.pos[1]);
            ticks += 1;
        }

        // Analytic 2*14/30 = 0.93s (56s); this stepping order lands a bit
        // later
        assert!((55..=59).contains(&ticks), "landed after {ticks} ticks");

        // Analytic 14^2/(2*30) = 3.27m; discrete stepping overshoots
        // slightly
        assert!((3.2..=3.5).contains(&apex), "apex was {apex}");
    }

    #[test]
    fn mv_y_jump_ignored_in_air() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        p.pos[1] = FLOOR_Y + 5.0;
        p.is_on_floor = false;

        let inp = Input {
            jump: true,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();

        // Only gravity applied: -30 * (1/60)
        assert!(approx(p.vel[1], -0.5));
    }

    #[test]
    fn mv_y_crouch_takes_ten_ticks() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            crouch: true,
            ..Default::default()
        };

        for _ in 0..9 {
            Step::new(&mut p, &inp).run();
        }
        assert!(p.height > CROUCH_HEIGHT + 0.05);

        Step::new(&mut p, &inp).run();
        assert!(approx(p.height, CROUCH_HEIGHT));

        // Further cruching can't go below the minimum
        Step::new(&mut p, &inp).run();
        assert!(approx(p.height, CROUCH_HEIGHT))
    }

    #[test]
    fn mv_y_crouch_release_regrows() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            crouch: true,
            ..Default::default()
        };
        for _ in 0..15 {
            Step::new(&mut p, &inp).run();
        }

        for _ in 0..15 {
            Step::new(&mut p, &Input::default()).run();
        }

        assert!(approx(p.height, HEIGHT));
    }

    #[test]
    fn mv_y_crouch_ignored_in_air() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        p.pos[1] = FLOOR_Y + 5.0;
        p.is_on_floor = false;
        let inp = Input {
            crouch: true,
            ..Default::default()
        };

        Step::new(&mut p, &inp).run();

        assert!(approx(p.height, HEIGHT));
    }
}
