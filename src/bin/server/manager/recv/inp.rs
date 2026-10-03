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
    buf: &'i [u8],
}

impl<'i, 'c, 'w> InpPkt<'i, 'c, 'w> {
    pub fn new(manager: &'i mut Manager<'c, 'w>, skt_src: SocketAddr, buf: &'i [u8]) -> Self {
        InpPkt {
            manager,
            skt_src,
            buf,
        }
    }

    pub fn recv(&mut self) {
        if !self.manager.codec.schema().mask_fits(self.buf) {
            return;
        }

        let (seq, inp) = self.inp();
    }

    fn inp(&mut self) -> (u8, Input) {
        let mut reader = self.manager.codec.reader(self.buf);

        let seq = reader.read(&FieldName::InputSeq) as u8;
        let mv_x = MV.decode(reader.read(&FieldName::InputMvX));
        let mv_z = MV.decode(reader.read(&FieldName::InputMvZ));
        let yaw = YAW.decode(reader.read(&FieldName::InputYaw));
        let jump = reader.read(&FieldName::InputJump) == 1;
        let crouch = reader.read(&FieldName::InputCrouch) == 1;

        (seq, Input::new(mv_x, mv_z, yaw, jump, crouch))
    }
}
