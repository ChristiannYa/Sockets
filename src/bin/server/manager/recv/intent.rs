use crate::manager::Manager;
use bitp::{
    pkt::FieldName,
    quantize::intent::{MV, YAW},
    sim::{Edges, Held, PlayerIntentSim},
    util::Seq,
};
use std::net::SocketAddr;

const INTENT_FIELDS: [FieldName; 6] = [
    FieldName::IntentSeq,
    FieldName::IntentMvX,
    FieldName::IntentMvZ,
    FieldName::IntentYaw,
    FieldName::IntentJump,
    FieldName::IntentCrouch,
];

pub struct IntentPkt<'i, 'c, 'w> {
    manager: &'i mut Manager<'c, 'w>,
    skt_src: SocketAddr,
    buf: &'i [u8],
    sid: u32,
}

impl<'i, 'c, 'w> IntentPkt<'i, 'c, 'w> {
    pub fn new(
        manager: &'i mut Manager<'c, 'w>,
        skt_src: SocketAddr,
        buf: &'i [u8],
        sid: u32,
    ) -> Self {
        IntentPkt {
            manager,
            skt_src,
            buf,
            sid,
        }
    }

    pub fn recv(&mut self) {
        let Some((seq, sim_intent)) = self.decode() else {
            return;
        };
        self.manager
            .world
            .players
            .queue_intent(self.sid, seq, sim_intent);
    }

    fn decode(&self) -> Option<(Seq, PlayerIntentSim)> {
        let schema = self.manager.codec.schema();

        let payload_ceil_len = INTENT_FIELDS
            .iter()
            .map(|field_name| schema.info_of(field_name).len)
            .sum::<usize>()
            .div_ceil(8);
        if self.buf.len() < schema.mask_len() + payload_ceil_len {
            return None;
        }

        let mut reader = self.manager.codec.reader(self.buf);
        if !INTENT_FIELDS
            .iter()
            .all(|field_name| reader.isset(field_name))
        {
            return None;
        }

        let seq = reader.read(&FieldName::IntentSeq) as u8;

        let held = Held {
            mv_x: MV.decode(reader.read(&FieldName::IntentMvX)),
            mv_z: MV.decode(reader.read(&FieldName::IntentMvZ)),
            yaw: YAW.decode(reader.read(&FieldName::IntentYaw)),
            crouch: reader.read(&FieldName::IntentCrouch) == 1,
        };

        let edges = Edges {
            jump: reader.read(&FieldName::IntentJump) == 1,
        };

        Some((Seq(seq), PlayerIntentSim::new(held, edges)))
    }
}
