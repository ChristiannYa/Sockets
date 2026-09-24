use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// Cutoff for how long a seen id is remembered before forgotten.
/// Longer than the retry max window to allow for one las in-flight duplicate
/// that was already on the wire when the sender gave up.
pub const SEEN_CUTOFF: Duration = Duration::from_secs(2);

/// Tracks ids already processed from a single sender, so a retransmitted
/// duplicate isn't applied twice.
/// Not scoped to a sender itself.
pub struct SeenPacketIds {
    ids: HashMap<u8, Instant>,
}

impl SeenPacketIds {
    pub fn new() -> Self {
        SeenPacketIds {
            ids: HashMap::new(),
        }
    }

    pub fn is_seen(&self, id: u8) -> bool {
        self.ids.contains_key(&id)
    }

    pub fn mark_seen(&mut self, id: u8, now: Instant) {
        self.ids.insert(id, now);
    }

    /// Removes entries older than `SEEN_CUTOFF`
    pub fn sweep(&mut self, now: Instant) {
        self.ids
            .retain(|_, seen_at| now.duration_since(*seen_at) < SEEN_CUTOFF);
    }
}

impl Default for SeenPacketIds {
    fn default() -> Self {
        Self::new()
    }
}
