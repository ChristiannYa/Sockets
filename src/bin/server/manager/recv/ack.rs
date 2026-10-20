use std::net::SocketAddr;

use crate::manager::Manager;

pub fn recv(manager: &mut Manager, skt_src: SocketAddr, buf: &[u8]) {
    if let Some(id) = bitp::rel::ack::decode(buf) {
        crate::rel::ack::handle_ack(manager.pending_pkts, id, &skt_src);
    }
}
