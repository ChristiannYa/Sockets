use crate::manager::Manager;
use bitp::rel::retx::RetryAction;
use std::{net::SocketAddr, time::Instant};

pub fn retry(manager: &mut Manager) {
    let mut giveups = Vec::<(u8, SocketAddr)>::new();
    let now = Instant::now();

    for ((id, addr), pkt) in manager.pending_pkts.iter_mut() {
        match pkt.retry_action(now) {
            RetryAction::Wait => {}
            RetryAction::Resend => {
                manager.net.send_to(&pkt.bytes, *addr);
                pkt.on_resend(now);
            }
            RetryAction::GiveUp => giveups.push((*id, *addr)),
        }
    }

    for key in giveups {
        manager.pending_pkts.remove(&key);
    }

    manager.addrs_seen_pkt_ids.sweep_all(now);
}
