use std::{collections::HashMap, net::SocketAddr, time::Instant};

use bitp::rel::dedup::SeenPacketIds;

/// One `SeendIds` set per client because each client runs its own id counter
pub struct AddrsSeenPacketIds {
    ids: HashMap<SocketAddr, SeenPacketIds>,
}

impl AddrsSeenPacketIds {
    pub fn new() -> Self {
        AddrsSeenPacketIds {
            ids: HashMap::new(),
        }
    }

    /// Checks whether `id` from `skt_src` has already been processed.
    pub fn is_seen(&self, skt_src: &SocketAddr, id: u8) -> bool {
        self.ids.get(skt_src).is_some_and(|ids| ids.is_seen(id))
    }

    /// Marks `id` from `skt_src` as seen
    pub fn mark_seen(&mut self, skt_src: &SocketAddr, id: u8, now: Instant) {
        self.ids.entry(*skt_src).or_default().mark_seen(id, now);
    }

    /// Sweeps every sender's set
    // @TODO: remove empty per-client entries once their set empties out
    pub fn sweep_all(&mut self, now: Instant) {
        for seen_ids in self.ids.values_mut() {
            seen_ids.sweep(now);
        }
    }
}
