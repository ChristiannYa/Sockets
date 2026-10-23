mod frz_wtch;

use frz_wtch::FreezeWatch;

#[derive(Default)]
pub struct PlayersStateDbg {
    pub frzw: FreezeWatch,
}
