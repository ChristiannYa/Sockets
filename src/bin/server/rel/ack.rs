use std::net::SocketAddr;

use crate::rel::retx::PendingPackets;

/// Handles a received ack by removing the matching pending entry, but
/// only if `skt_src` matches the address it was originally sent to.
///
/// Returns `true` if an entry was removed.
pub fn handle_ack(pending_pkts: &mut PendingPackets, id: u8, skt_src: &SocketAddr) -> bool {
    pending_pkts.remove(&(id, *skt_src)).is_some()
}
