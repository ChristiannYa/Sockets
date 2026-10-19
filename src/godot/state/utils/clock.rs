//! Puts the server's snapshot ticks on the local clock.

/// Seconds between two server ticks
const TICK_DT: f64 = 1.0 / 60.0;

pub struct SnapshotClock {
    /// Newest tick seen, counted past the 2-byte wrap. `None` until the first
    /// snapshot arrives.
    tick: Option<i64>,
    /// Best estimate so far of (local clock - server clock), in seconds
    offset: f64,
}

impl SnapshotClock {
    pub fn new() -> Self { SnapshotClock { tick: None, offset: 0.0 } }

    /// Takes a snapshot's tick and the time it arrived.
    ///
    /// Returns the snapshot's time on the local clock, or `None` when it is
    /// not newer than the last one (a duplicate or a reordered packet) and
    /// should be dropped.
    ///
    /// The tick is a 2-byte number treated like a clock face, so 65535 -> 0
    /// counts as one tick forward.
    pub fn accept(&mut self, tick: u16, arrival_s: f64) -> Option<f64> {
        let tick = match self.tick {
            None => i64::from(tick),
            Some(newest) => {
                let ahead = tick.wrapping_sub(newest as u16) as i16;
                if ahead <= 0 {
                    return None;
                }
                newest + i64::from(ahead)
            }
        };

        // A late packet makes the estimate larger, so only the lowest one is
        // kept
        let est = arrival_s - tick as f64 * TICK_DT;
        self.offset =
            if self.tick.is_none() { est } else { self.offset.min(est) };
        self.tick = Some(tick);

        Some(tick as f64 * TICK_DT + self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::{SnapshotClock, TICK_DT};

    #[test]
    fn first_snapshot_is_accepted() {
        let mut c = SnapshotClock::new();
        assert!(c.accept(7, 1.0).is_some());
    }

    #[test]
    fn duplicates_and_older_are_dropped() {
        let mut c = SnapshotClock::new();
        assert!(c.accept(10, 1.0).is_some());
        assert!(c.accept(10, 1.1).is_none());
        assert!(c.accept(9, 1.1).is_none());
        assert!(c.accept(11, 1.2).is_some());
    }

    #[test]
    fn counts_past_the_wrap() {
        let mut c = SnapshotClock::new();
        let a = c.accept(65535, 100.0).unwrap();
        let b = c.accept(0, 100.0 + TICK_DT).unwrap();
        assert!((b - a - TICK_DT).abs() < 1e-9);
    }

    #[test]
    fn a_late_packet_does_not_push_time_later() {
        let mut c = SnapshotClock::new();
        c.accept(0, 10.0);
        // arrives 50 ms later than its tick says
        let t = c.accept(1, 10.0 + TICK_DT + 0.05).unwrap();
        assert!((t - (10.0 + TICK_DT)).abs() < 1e-9);
    }
}
