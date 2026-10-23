use super::PoseSample;
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

    /// Adds a sample taken at `time_s`.
    ///
    /// Times must strictly increase: a sample that is not newer than the
    /// newest stored one (stale or duplicate) is ignored.
    ///
    /// When full, the oldest sample is dropped to make room.
    ///
    /// ```text
    /// stored times: 1.0, 2.0
    /// push 3.0 -> stored: 1.0, 2.0, 3.0
    /// push 2.5 -> ignored: (older than 3.0)
    /// push 3.0 -> ignored: (same time as 3.0)
    /// ```
    pub fn push(&mut self, time_s: f64, pose: PoseSample) {
        // self.samples.back() id newest stored sample and returns None while
        // the history is empty.
        //
        // is_some_and(...) is `true` when a newest sample exists and the
        // time_s is not strictly later than t
        if self.samples.back().is_some_and(|(t, _)| time_s <= *t) {
            return; // stale or suplicate
        }

        if self.samples.len() >= self.cap {
            self.samples.pop_front();
        }
        self.samples.push_back((time_s, pose));
    }

    /// Pose at `t_s`, or [`None`] while the history is empty.
    ///
    /// If `t_s` falls between two stored samples the pose is blended in
    /// proportion to how far `t_s` is between them.
    /// Before the oldest or after the newest sample there is nothing to blend
    /// with, so that sample's pose is returned as is (no extrapolation).
    ///
    /// ```text
    /// samples: t=1.0 x=10, t=2.0 x=20
    /// at(0.0) -> x=10 (before the oldest: oldest sample)
    /// at(1.5) -> x=15 (halfway between the two)
    /// at(5.0) -> x=20 (after the newest: newest sample)
    /// ```
    pub fn at(&self, t_s: f64) -> Option<PoseSample> {
        let &(f_t, f_p) = self.samples.front()?;
        let &(b_t, b_p) = self.samples.back()?;

        if t_s <= f_t {
            return Some(f_p); // clamp: no extrapolation
        }
        if t_s >= b_t {
            return Some(b_p);
        }

        // Find the pair of samples that bracket t_s, so that
        // lo_t < t_s <= hi_t.
        //
        // hi_i is the index of the first sample at or after t_s (the
        // partition point), and lo_i = hi_i - 1 is the sample before it.
        // Both are valid because the early returns above guarantee
        // f_t < t_s < b_t, so hi_i is neither 0 nor past the end.
        //
        // Ex:
        // times = [1.0, 2.0, 3.0]
        // t_s   = 2.5
        // hi_i  = 2 -> (lo = (2.0, _), hi = (3.0, _))
        //
        // times = [1.0, 2.0, 3.0, 5.0, 8.0]
        // t_s   = 5.0
        // hi_i  = 3 -> (lo = (3.0, _), hi = (5.0, _))
        let hi_i = self.samples.partition_point(|(t, _)| *t < t_s);
        let (hi_t, hi_p) = self.samples[hi_i];
        let (lo_t, lo_p) = self.samples[hi_i - 1];

        // hi_t > lo_t is guaranteed since `push` only accepts
        // strictly increasing times, so the division below is safe.
        let w = ((t_s - lo_t) / (hi_t - lo_t)) as f32;

        Some(PoseSample::blend(lo_p, hi_p, w))
    }

    /// Newest stored sample, or [`None`] while empty.
    pub fn latest(&self) -> Option<PoseSample> {
        self.samples.back().map(|&(_, p)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(x: f32, is_crouching: bool) -> PoseSample {
        PoseSample { x, y: 0.0, z: 0.0, yaw: 0.0, is_crouching }
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

    #[test]
    fn latest_is_newest_sample() {
        assert!(PoseHistory::new(4).latest().is_none());

        let h = history(&[(1.0, pose(10.0, false)), (2.0, pose(20.0, true))]);
        assert_eq!(h.latest().unwrap().x, 20.0);
    }
}
