mod data;
mod intent;

use crate::manager::{
    Manager,
    recv::{data::DataPkt, intent::IntentPkt},
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
        Ok(PacketKind::Intent) => {
            let Some(sid) = manager.sess.id_of(&skt_src) else {
                return;
            };
            if let Some(buf) = buf.get(1..) {
                IntentPkt::new(manager, skt_src, buf, sid).recv();
            }
        }
        Err(()) => {}
    }
}
