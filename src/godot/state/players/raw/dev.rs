//! Merge does not apply for this module: merge is for state that persists
//! accross records, like pose and color, where "keep the old values if the
//! record lacks it" is fine. Dev fields describe one record:
//!
//! - `sid`: it identifies a record. It is a key rather than something to
//!   merge
//! - `simed_seq`: if it were merged and a record lacked it, you'd get the
//!   previous record's value. The reconcile would then trust a stale ack
//! - `im_new`: it is an event, not a state. If merged it would stay `true`
//!   forever

use crate::{
    godot::state::players::PlayerDev,
    pkt::{FieldName, Seq},
};

#[derive(Default)]
pub struct RawDev {
    sid: Option<u32>,
    seq: Option<u32>,
    simed_seq: Option<u32>,
    im_new: Option<u32>,
}

impl RawDev {
    pub fn set(&mut self, name: &FieldName, val: u32) {
        match name {
            FieldName::DevSid => self.sid = Some(val),
            FieldName::DevSeq => self.seq = Some(val),
            FieldName::DevSimedSeq => self.simed_seq = Some(val),
            FieldName::DevImNew => self.im_new = Some(val),
            _ => {}
        };
    }

    pub fn decode(&self) -> Option<PlayerDev> {
        Some(PlayerDev {
            sid: self.sid?,
            seq: self.seq.map(|v| Seq(v as u8)),
            simed_seq: self.simed_seq.map(|v| Seq(v as u8)),
            im_new: self.im_new == Some(1),
        })
    }
}
