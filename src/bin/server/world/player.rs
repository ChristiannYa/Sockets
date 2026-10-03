use bitp::sim::{Input, PlayerState};
use std::collections::VecDeque;

pub struct Player {
    pub state: PlayerState,
    inps: VecDeque<(u8, Input)>,
    latest_seq: Option<u8>,
    last_inp: Input,

    /// Sequence number of the last input the tick actually used
    pub applied_seq: u8,
}
