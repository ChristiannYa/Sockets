mod ss_wtch;

use super::GAP_WARN_S;
use ss_wtch::StateSnapshotWatch;

#[derive(Default)]
pub struct StateDebug {
    pub ss_wtch: StateSnapshotWatch,
}
