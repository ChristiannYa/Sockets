use std::net::SocketAddr;

use crate::manager::Manager;

pub struct AckPkt<'s, 'm, 'w> {
    manager: &'s mut Manager<'m, 'w>,
    buf: &'s [u8],
    skt_src: SocketAddr,
}

impl<'s, 'm, 'w> AckPkt<'s, 'm, 'w> {
    pub fn new(
        manager: &'s mut Manager<'m, 'w>,
        buf: &'s [u8],
        skt_src: SocketAddr,
    ) -> AckPkt<'s, 'm, 'w> {
        AckPkt { manager, buf, skt_src }
    }

    pub fn recv(&mut self) {
        if let Some(id) = bitp::rel::ack::decode(self.buf) {
            crate::rel::ack::handle_ack(
                self.manager.pending_pkts,
                id,
                &self.skt_src,
            );
        }
    }
}
