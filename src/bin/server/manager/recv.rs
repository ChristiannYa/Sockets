mod ack;
mod data;
mod intent;

use crate::manager::Manager;
use bitp::pkt::PacketKind;
use std::net::SocketAddr;

pub fn cli(manager: &mut Manager, buf: &[u8], skt_src: SocketAddr) {
    let Some(&kind) = buf.first() else {
        return;
    };

    match PacketKind::try_from(kind) {
        Ok(PacketKind::Data) => data::recv(manager, skt_src, buf),
        Ok(PacketKind::Ack) => ack::recv(manager, skt_src, buf),
        Ok(PacketKind::Intent) => intent::recv(manager, skt_src, buf),
        Ok(PacketKind::Snapshot) | Err(()) => {}
    }
}
