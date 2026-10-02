mod data;
mod inp;

use crate::manager::{
    Manager,
    recv::{data::DataPkt, inp::InpPkt},
};
use bitp::pkt::PacketKind;
use std::net::SocketAddr;

pub fn buf(manager: &mut Manager, buf: &[u8], skt_src: SocketAddr) {
    let Some(&kind) = buf.first() else { return };

    match PacketKind::try_from(kind) {
        Ok(PacketKind::Ack) => {
            if let Some(id) = bitp::rel::ack::decode(buf) {
                crate::rel::ack::handle_ack(manager.pending_pkts, id, &skt_src);
            }
        }
        Ok(PacketKind::Data) => {
            if let Some(buf) = buf.get(2..) {
                DataPkt::new(manager, skt_src, buf).recv();
            };
        }
        Ok(PacketKind::Inp) => {
            if let Some(buf) = buf.get(1..) {
                InpPkt::new(manager, skt_src, buf).recv();
            }
        }
        Err(()) => {}
    }
}
