use crate::sim::{
    consts::{FLOOR_Y, GRAVITY, TICK_DT},
    input::Input,
    state::PlayerState,
};

pub fn step(p: &mut PlayerState, _inp: &Input) {
    let dt = TICK_DT;

    // Gravity
    p.vel[1] -= GRAVITY * dt;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
