use crate::sim::{
    consts::{DECC, FLOOR_Y, GRAVITY, SPEED, TICK_DT},
    input::Input,
    state::PlayerState,
};

/// Advance player by one fixed tick.
pub fn step(p: &mut PlayerState, inp: &Input) {
    let dt = TICK_DT;

    // Gravity
    p.vel[1] -= GRAVITY * dt;

    // Horizontal velocity
    let (wd_x, wd_z) = wish_dir(inp);
    if (wd_x.powi(2) + wd_z.powi(2)).sqrt() > 0.01 {
        p.vel[0] = wd_x * SPEED;
        p.vel[2] = wd_z * SPEED;
    } else {
        p.vel[0] = move_toward(p.vel[0], 0.0, DECC * dt);
        p.vel[2] = move_toward(p.vel[2], 0.0, DECC * dt);
    }

    // Integrate
    p.pos[0] += p.vel[0] * dt;
    p.pos[1] += p.vel[1] * dt;
    p.pos[2] += p.vel[2] * dt;

    // Temporary flat floor
    if p.pos[1] <= FLOOR_Y {
        p.pos[1] = FLOOR_Y;
        p.vel[1] = 0.0;
        p.is_on_floor = true;
    } else {
        p.is_on_floor = false;
    }
}

fn wish_dir(inp: &Input) -> (f32, f32) {
    let mag = (inp.mv_x.powi(2) + inp.mv_z.powi(2)).sqrt();

    let (mv_x, mv_z) = if mag > 1.0 {
        (inp.mv_x / mag, inp.mv_z / mag)
    } else {
        (inp.mv_x, inp.mv_z)
    };

    let (yaw_sin, yaw_cos) = inp.yaw.sin_cos();
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
            step(&mut p, &inp);
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
            step(&mut p, &inp);
            ticks += 1;
        }

        // Free fall from 5m at g=30: sqrt(2*5/30) = 0.577s. ~35 ticks
        assert!((34..=37).contains(&ticks), "landed after {ticks} ticks");
        assert_eq!(p.pos[1], FLOOR_Y)
    }

    #[test]
    fn mv_fwd_inp_reaches_full_speed_immediately() {
        let mut p = PlayerState::spawn(0.0, 0.0);

        // In Godot, "forward" is -z
        let inp = Input {
            mv_z: -1.0,
            ..Default::default()
        };

        step(&mut p, &inp);

        assert!(approx(p.vel[2], -6.0));
        assert!(approx(p.pos[2], -6.0 * TICK_DT));
    }

    #[test]
    fn mv_yaw_rotates_movement() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            mv_z: -1.0,
            yaw: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };

        step(&mut p, &inp);

        // Facing +90deg around y, "forward" points along -x
        assert!(approx(p.vel[0], -6.0));
        assert!(approx(p.vel[2], 0.0));
    }

    #[test]
    fn mv_diagonal_input_is_clamped() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            mv_x: 1.0,
            mv_z: 1.0,
            ..Default::default()
        };

        step(&mut p, &inp);

        let speed = (p.vel[0].powi(2) + p.vel[2].powi(2)).sqrt();
        assert!(approx(speed, 6.0), "speed was {speed}");
    }

    #[test]
    fn mv_inp_release_stops_in_about_a_third_of_a_second() {
        let mut p = PlayerState::spawn(0.0, 0.0);
        let inp = Input {
            mv_z: -1.0,
            ..Default::default()
        };

        step(&mut p, &inp);

        let mut ticks = 0;
        while p.vel[2] != 0.0 && ticks < 100 {
            step(&mut p, &Input::default());
            ticks += 1;
        }

        // 6 / 20 = 0.3s = 18 ticks
        assert!((17..=19).contains(&ticks), "stopped after {ticks} ticks");
    }
}
