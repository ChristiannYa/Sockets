mod recv;
mod retry;
mod sim;

use crate::{
    codec::Codec,
    events::Event,
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
        Manager { net, sess, codec, world, pending_pkts, addrs_seen_pkt_ids }
    }

    pub fn recv(&mut self, ev: Event) {
        match ev {
            Event::Retry => retry::retry(self),
            Event::Client(buf, skt_src) => recv::cli(self, &buf, skt_src),
            Event::SimTick => sim::tick(self),
        }
    }
}
