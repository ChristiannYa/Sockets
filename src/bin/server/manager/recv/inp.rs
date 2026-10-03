use std::net::SocketAddr;

use crate::manager::Manager;
use bitp::{
    pkt::FieldName,
    quantize::inp::{MV, YAW},
    sim::{Edges, Held, PlayerInpSim},
};

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
        if !self.manager.codec.schema().mask_fits(self.buf) {
            return;
        }

        let (seq, sim_inp) = self.inp();
    }

    fn inp(&mut self) -> (u8, PlayerInpSim) {
        let mut reader = self.manager.codec.reader(self.buf);

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

        (seq, PlayerInpSim::new(held, edges))
    }
}
