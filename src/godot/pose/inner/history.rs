use super::sample::PoseSample;
use std::collections::VecDeque;

/// Time-ordered pose samples of one player, oldest first
pub struct PoseHistory {
    cap: usize,
    samples: VecDeque<(f64, PoseSample)>,
}

impl PoseHistory {
    pub fn new(cap: usize) -> Self {
        PoseHistory { cap, samples: VecDeque::new() }
    }

    pub fn push(&mut self, time_s: f64, pose: PoseSample) {
        if self.samples.back().is_some_and(|(t, _)| time_s <= *t) {
            return; // stale or suplicate
        }

        if self.samples.len() >= self.cap {
            self.samples.pop_front();
        }
        self.samples.push_back((time_s, pose));
    }

    /// Hold the nearest sample outside the stored range, otherwise blend the
    /// two samples around it. `None` while empty.
    pub fn at(&self, at_t: f64) -> Option<PoseSample> {
        let &(oldest_t, oldest) = self.samples.front()?;
        let &(newest_t, newest) = self.samples.back()?;

        if at_t <= oldest_t {
            return Some(oldest);
        }
        if at_t >= newest_t {
            return Some(newest);
        }

        // First index with time >= at_t. It is >= 1 since at_t > oldest_t
        let i = self.samples.partition_point(|(t, _)| *t < at_t);
        let (older_t, older) = self.samples[i - 1];
        let (newer_t, newer) = self.samples[i];

        let elapsed = newer_t - older_t;
        let w = if elapsed > 0.0 {
            ((at_t - older_t) / elapsed) as f32
        } else {
            1.0
        };

        Some(PoseSample::blend(older, newer, w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(x: f32, crouch: bool) -> PoseSample {
        PoseSample { x, y: 0.0, z: 0.0, yaw: 0.0, is_crouching: crouch }
    }

    fn history(items: &[(f64, PoseSample)]) -> PoseHistory {
        let mut h = PoseHistory::new(16);
        for &(t, p) in items {
            h.push(t, p);
        }
        h
    }

    #[test]
    fn empty_is_none() {
        assert!(PoseHistory::new(4).at(1.0).is_none());
    }

    #[test]
    fn holds_nearest_outside_range() {
        let h = history(&[(1.0, pose(10.0, false)), (2.0, pose(20.0, true))]);
        assert_eq!(h.at(0.0).unwrap().x, 10.0);
        assert_eq!(h.at(5.0).unwrap().x, 20.0);
    }

    #[test]
    fn blends_between_samples() {
        let h = history(&[
            (1.0, pose(10.0, false)),
            (2.0, pose(20.0, true)),
            (3.0, pose(40.0, true)),
        ]);

        let p = h.at(1.5).unwrap();
        assert!((p.x - 15.0).abs() < 1e-4);
        assert!(!p.is_crouching); // older value held

        let p = h.at(2.5).unwrap();
        assert!((p.x - 30.0).abs() < 1e-4);
        assert!(p.is_crouching);
    }

    #[test]
    fn drops_oldest_past_capacity() {
        let mut h = PoseHistory::new(2);
        h.push(1.0, pose(1.0, false));
        h.push(2.0, pose(2.0, false));
        h.push(3.0, pose(3.0, false));
        assert_eq!(h.at(0.0).unwrap().x, 2.0);
    }
}
