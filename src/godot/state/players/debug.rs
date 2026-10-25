mod frz_wtch;

use super::GAP_WARN_S;
use frz_wtch::FreezeWatch;

#[derive(Default)]
pub struct PlayersStateDbg {
    pub frzw: FreezeWatch,
}
