use super::GAP_WARN_S;
use crate::logt;

#[derive(Default)]
pub struct StateSnapshotWatch {
    /// (arrival_s, snapshot_s)
    last_accs_ss: Option<(f64, f64)>,
}

impl StateSnapshotWatch {
    /// Logs when too much time passed between snapshot arrivals. Compare the
    /// two numbers in the line:
    /// - about equal: the snapshots in between never arrived (lost, or never
    ///   sent)
    /// - arrival gap big, server time about one tick: the next snapshot came
    ///   late (delay or WiFi stall, not loss)
    pub fn track_gap(&mut self, arrival_s: f64, ss_s: f64) {
        if let Some((prev_s, prev_ss)) = self.last_accs_ss {
            let arrival_gap = arrival_s - prev_s;
            if arrival_gap > GAP_WARN_S {
                let ss_gap = ss_s - prev_ss;
                logt!(
                    "ss gap: nothing arrived for {:.0}ms (server time between: {:.0}ms)",
                    arrival_gap * 1000.0,
                    ss_gap * 1000.0
                );
            }
        }

        self.last_accs_ss = Some((arrival_s, ss_s));
    }
}
