use crate::manager::Manager;
use bitp::{
    pkt::FieldName,
    quantize::inp::{MV, YAW},
    sim::{Edges, Held, PlayerInpSim},
    util::Seq,
};
use std::net::SocketAddr;

const INP_FIELDS: [FieldName; 6] = [
    FieldName::InputSeq,
    FieldName::InputMvX,
    FieldName::InputMvZ,
    FieldName::InputYaw,
    FieldName::InputJump,
    FieldName::InputCrouch,
];

pub struct InpPkt<'i, 'c, 'w> {
    manager: &'i mut Manager<'c, 'w>,
    skt_src: SocketAddr,
    buf: &'i [u8],
    sid: u32,
}

impl<'i, 'c, 'w> InpPkt<'i, 'c, 'w> {
    pub fn new(
        manager: &'i mut Manager<'c, 'w>,
        skt_src: SocketAddr,
        buf: &'i [u8],
        sid: u32,
    ) -> Self {
        InpPkt {
            manager,
            skt_src,
            buf,
            sid,
        }
    }

    pub fn recv(&mut self) {
        let Some((seq, sim_inp)) = self.inp() else {
            return;
        };
        self.manager.world.players.push_inp(self.sid, seq, sim_inp);
    }

    fn inp(&self) -> Option<(Seq, PlayerInpSim)> {
        let schema = self.manager.codec.schema();

        let payload_ceil_len = INP_FIELDS
            .iter()
            .map(|field_name| schema.info_of(field_name).len)
            .sum::<usize>()
            .div_ceil(8);
        if self.buf.len() < schema.mask_len() + payload_ceil_len {
            return None;
        }

        let mut reader = self.manager.codec.reader(self.buf);
        if !INP_FIELDS.iter().all(|field_name| reader.isset(field_name)) {
            return None;
        }

        let seq = reader.read(&FieldName::InputSeq) as u8;

        let held = Held {
            mv_x: MV.decode(reader.read(&FieldName::InputMvX)),
            mv_z: MV.decode(reader.read(&FieldName::InputMvZ)),
            yaw: YAW.decode(reader.read(&FieldName::InputYaw)),
            crouch: reader.read(&FieldName::InputCrouch) == 1,
        };

        let edges = Edges {
            jump: reader.read(&FieldName::InputJump) == 1,
        };

        Some((Seq(seq), PlayerInpSim::new(held, edges)))
    }
}
