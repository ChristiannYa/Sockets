mod ack;
mod data;
mod intent;

use crate::manager::{
    Manager,
    recv::{ack::AckPkt, data::DataPkt, intent::IntentPkt},
};
use bitp::pkt::PacketKind;
use std::net::SocketAddr;

pub fn cli(manager: &mut Manager, buf: &[u8], skt_src: SocketAddr) {
    let Some(&kind) = buf.first() else {
        return;
    };

    match PacketKind::try_from(kind) {
        Ok(PacketKind::Ack) => AckPkt::new(manager, buf, skt_src).recv(),
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
                IntentPkt::new(manager, buf, sid).recv();
            }
        }
        Ok(PacketKind::Snapshot) | Err(()) => {}
    }
}
