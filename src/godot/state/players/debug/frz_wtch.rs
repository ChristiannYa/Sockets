//! Debug logging for freezes: notices when snapshots stop arriving. Nothing
//! here changes what the game sees.

use super::GAP_WARN_S;
use crate::{logt, sim::consts::SS_INTV_S};
use std::time::Instant;

/// How far the render time must be past the newest snapshot to count as a
/// freeze (~1 snapshot interval): 20ms at 60hz, 60ms at 20hz
const FREEZE_MIN_S: f64 = 1.2 * SS_INTV_S;

#[derive(Default)]
pub struct FreezeWatch {
    /// Time of the newest snapshot applied, on the local clock
    newest_ss_s: Option<f64>,

    /// Render time when the current freeze began. [`None`] while smooth.
    frozen_since_s: Option<f64>,

    /// When the newest snapshot was applied
    last_ss_at: Option<Instant>,

    /// When [`Self::on_latest`] last ran, which is about once per frame
    last_frame_at: Option<Instant>,

    /// A STALL line was already logged for the current silence
    stall_logged: bool,
}

impl FreezeWatch {
    /// A snapshot was just applied. `t_s` is its time on the local clock.
    pub fn on_ss(&mut self, t_s: f64) {
        self.newest_ss_s = Some(t_s);
        self.last_ss_at = Some(Instant::now());
        self.stall_logged = false;
    }

    /// Logs once when the render time passes the newest snapshot (no more
    /// data to play) and once when it recovers.
    pub fn on_sample(&mut self, t_s: f64) {
        let Some(newest_ss_s) = self.newest_ss_s else {
            return;
        };
        let past_s = t_s - newest_ss_s;

        match (self.frozen_since_s, past_s > FREEZE_MIN_S) {
            (None, true) => {
                self.frozen_since_s = Some(t_s);
                logt!(
                    "FREEZE: render {:.0}ms past the newest",
                    past_s * 1000.0
                );
            }
            (Some(since), false) => {
                self.frozen_since_s = None;
                logt!("UNFROZEN: after {:.0}ms", (t_s - since) * 1000.0);
            }
            _ => {}
        }
    }

    /// Logs once, while it is happening, when no snapshot has arrived for
    /// longer than [`GAP_WARN_S`]. Meant to be called once per frame (the
    /// local player's `latest`), so this fires mid-freeze instead of after it.
    pub fn on_latest(&mut self) {
        let now = Instant::now();
        let frame_gap_s = self
            .last_frame_at
            .replace(now)
            .map(|prev| now.duration_since(prev).as_secs_f64());

        let Some(ss_at) = self.last_ss_at else {
            return;
        };
        let since_ss_s = now.duration_since(ss_at).as_secs_f64();

        if since_ss_s > GAP_WARN_S && !self.stall_logged {
            self.stall_logged = true;
            logt!(
                "STALL: no ss for {:.0}ms and counting (last frame took {:.0}ms)",
                since_ss_s * 1000.0,
                frame_gap_s.unwrap_or(0.0) * 1000.0
            );
        }
    }
}
