use crate::sim::{PlayerState, consts::DIAM};

const PASSES: usize = 4;
const EPS: f32 = 1e-4;

pub struct Collision<'c> {
    ps: Vec<&'c mut PlayerState>,
}

impl<'c> Collision<'c> {
    pub fn new(ps: Vec<&'c mut PlayerState>) -> Self { Collision { ps } }

    pub fn resolve(&mut self) {
        for _ in 0..PASSES {
            let mut moved = false;

            for j in 1..self.ps.len() {
                let (head, tail) = self.ps.split_at_mut(j);
                let p_b = &mut *tail[0];
                for p_a in head.iter_mut() {
                    moved |= split(p_a, p_b)
                }
            }

            if !moved {
                break;
            }
        }
    }
}

/// Pushes 2 overlapping players apart along the x/z line between them.
/// Returns wether anything moved.
fn split(p_a: &mut PlayerState, p_b: &mut PlayerState) -> bool {
    // No vertical overlap (feet are pos[1], top is pos[1] + height): one
    // player is above the other, so they don't collide
    if p_a.pos[1] >= p_b.pos[1] + p_b.height
        || p_b.pos[1] >= p_a.pos[1] + p_a.height
    {
        return false;
    }

    let (del_x, del_z) = (p_b.pos[0] - p_a.pos[0], p_b.pos[2] - p_a.pos[2]);
    let dist = del_x.hypot(del_z);
    if dist >= DIAM - EPS {
        return false;
    }

    // Excatly on top of each other: fixed direction, never random
    let (n_x, n_z) =
        if dist > EPS { (del_x / dist, del_z / dist) } else { (1.0, 0.0) };
    let push = (DIAM - dist) * 0.5;

    p_a.pos[0] -= n_x * push;
    p_a.pos[2] -= n_z * push;
    p_b.pos[0] += n_x * push;
    p_b.pos[2] += n_z * push;

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::consts::RADIUS;

    fn dist(p_a: &PlayerState, p_b: &PlayerState) -> f32 {
        (p_b.pos[0] - p_a.pos[0]).hypot(p_b.pos[2] - p_a.pos[2])
    }

    #[test]
    fn overlapping_pair_is_separated_symmetrically() {
        let mut p_a = PlayerState::spawn(0.0, 0.0);
        let mut p_b = PlayerState::spawn(0.4, 0.0);

        Collision::new(vec![&mut p_a, &mut p_b]).resolve();

        assert!((dist(&p_a, &p_b) - 2.0 * RADIUS).abs() < 1e-3);
        // Midpoint (0.2) is preserved: both moved by the same amount
        assert!((p_a.pos[0] + p_b.pos[0] - 0.4).abs() < 1e-3);
    }

    #[test]
    fn non_overlapping_pair_is_untouched() {
        let mut p_a = PlayerState::spawn(0.0, 0.0);
        let mut p_b = PlayerState::spawn(3.0, 0.0);

        Collision::new(vec![&mut p_a, &mut p_b]).resolve();

        assert_eq!(p_a.pos, [0.0, 0.0, 0.0]);
        assert_eq!(p_b.pos, [3.0, 0.0, 0.0]);
    }

    #[test]
    fn exact_overlap_is_deterministic() {
        let mut p_a = PlayerState::spawn(0.0, 0.0);
        let mut p_b = PlayerState::spawn(0.0, 0.0);

        Collision::new(vec![&mut p_a, &mut p_b]).resolve();

        assert!((p_a.pos[0] + RADIUS).abs() < 1e-3);
        assert!((p_b.pos[0] - RADIUS).abs() < 1e-3);
    }

    #[test]
    fn players_at_different_heights_dont_collide() {
        let mut p_a = PlayerState::spawn(0.0, 0.0);
        let mut p_b = PlayerState::spawn(0.1, 0.0);
        p_b.pos[1] = 5.0; // well above a's head

        Collision::new(vec![&mut p_a, &mut p_b]).resolve();

        assert_eq!(p_a.pos[0], 0.0);
        assert_eq!(p_b.pos[0], 0.1);
    }
}
