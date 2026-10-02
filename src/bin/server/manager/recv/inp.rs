use std::net::SocketAddr;

use crate::manager::Manager;
use bitp::{
    pkt::FieldName,
    quantize::input::{MV, YAW},
    sim::input::Input,
};

pub struct InpPkt<'i, 'c, 'w> {
    manager: &'i mut Manager<'c, 'w>,
    skt_src: SocketAddr,
    seq: u8,
    inp: Input,
}

impl<'i, 'c, 'w> InpPkt<'i, 'c, 'w> {
    pub fn new(manager: &'i mut Manager<'c, 'w>, skt_src: SocketAddr, buf: &[u8]) -> Self {
        let mut reader = manager.codec.reader(buf);

        let seq = reader.read(&FieldName::InputSeq) as u8;
        let mv_x = MV.decode(reader.read(&FieldName::InputMvX));
        let mv_z = MV.decode(reader.read(&FieldName::InputMvZ));
        let yaw = YAW.decode(reader.read(&FieldName::InputYaw));
        let jump = reader.read(&FieldName::InputJump) == 1;
        let crouch = reader.read(&FieldName::InputCrouch) == 1;

        InpPkt {
            manager,
            skt_src,
            seq,
            inp: Input::new(mv_x, mv_z, yaw, jump, crouch),
        }
    }

    pub fn recv(&mut self) {}
}
