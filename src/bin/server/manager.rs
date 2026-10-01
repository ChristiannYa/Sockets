mod recv_cli;
mod retry;

use std::net::SocketAddr;

use crate::{
    codec::Codec,
    net::Net,
    rel::{dedup::AddrsSeenPacketIds, retx::PendingPackets},
    session::Session,
    world::World,
};

pub struct Manager<'c, 'w> {
    pub net: &'c Net,
    pub sess: &'c mut Session,
    pub codec: &'c Codec,
    pub world: &'c mut World<'w>,
    pub pending_pkts: &'c mut PendingPackets,
    pub addrs_seen_pkt_ids: &'c mut AddrsSeenPacketIds,
}

impl<'c, 'w> Manager<'c, 'w> {
    pub fn new(
        net: &'c Net,
        sess: &'c mut Session,
        codec: &'c Codec,
        world: &'c mut World<'w>,
        pending_pkts: &'c mut PendingPackets,
        addrs_seen_pkt_ids: &'c mut AddrsSeenPacketIds,
    ) -> Self {
        Manager {
            net,
            sess,
            codec,
            world,
            pending_pkts,
            addrs_seen_pkt_ids,
        }
    }

    pub fn retry(&mut self) {
        retry::retry(self);
    }

    pub fn recv_cli(&mut self, buf: &[u8], skt_src: SocketAddr) {
        recv_cli::recv_cli(self, buf, skt_src);
    }

    pub fn sim_tick(&mut self) {}
}
