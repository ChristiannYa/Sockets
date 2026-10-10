mod codec;
mod events;
mod manager;
mod net;
mod rel;
mod session;
mod world;

use std::time::Instant;

use crate::{
    codec::Codec,
    events::Event,
    manager::Manager,
    net::Net,
    rel::{dedup::AddrsSeenPacketIds, retx::PendingPackets},
    session::Session,
    world::World,
};

fn main() {
    let net = Net::build("0.0.0.0:34254");
    let evs_rx = events::spawn(net.try_clone_skt());

    let mut sess = Session::new();
    let codec = Codec::new();
    let mut world = World::new(&codec);

    let mut pending_pkts = PendingPackets::new();
    let mut addrs_seen_pkt_ids = AddrsSeenPacketIds::new();

    // @TODO: All events are handled by this thread, and it could cause
    // it to stall due slow work, which would slow down the events' threads
    for (queued_at, ev) in evs_rx {
        let queue_delay = queued_at.elapsed();
        let started = Instant::now();

        let mut manager = Manager::new(
            &net,
            &mut sess,
            &codec,
            &mut world,
            &mut pending_pkts,
            &mut addrs_seen_pkt_ids,
        );

        match ev {
            Event::Retry => manager.retry(),
            Event::Client(buf, skt_src) => manager.recv_cli(&buf, skt_src),
            Event::SimTick => manager.sim_tick(),
        }

        let handle_t = started.elapsed();
        if queue_delay.as_millis() >= 2 || handle_t.as_millis() >= 2 {
            eprintln!(
                "slow: queue_delay={queue_delay:?} handle_t={handle_t:?}"
            );
        }
    }
}
