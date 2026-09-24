use std::time::{Duration, Instant};

pub const RETRY_INTV: Duration = Duration::from_millis(100);
pub const RETRY_MAX_ATTEMPTS: u8 = 5;

pub enum RetryAction {
    Wait,
    Resend,
    GiveUp,
}

/// Splitting "decide" (`.retry_action(...)`) from "record that we resent" (`.on_resend(...)`)
/// matters because actually sending the bytes is I/O (this model should not reach into a socket)
pub struct PendingPacket {
    pub id: u8,
    pub bytes: Vec<u8>,
    pub last_sent: Instant,
    pub retries: u8,
}

impl PendingPacket {
    // Taking `now` as a parameter rather than calling `Instant::now()`
    // keeps the function testable
    pub fn new(id: u8, bytes: Vec<u8>, now: Instant) -> Self {
        PendingPacket {
            id,
            bytes,
            last_sent: now,
            retries: 0,
        }
    }
}

impl PendingPacket {
    /// Decides what to do with this entry on a retry tick, given the current
    /// time.
    ///
    /// Does not mutate or do anything, the caller will apply the result.
    pub fn retry_action(&self, now: Instant) -> RetryAction {
        match () {
            _ if now.duration_since(self.last_sent) < RETRY_INTV => RetryAction::Wait,
            _ if self.retries >= RETRY_MAX_ATTEMPTS => RetryAction::GiveUp,
            _ => RetryAction::Resend,
        }
    }

    pub fn on_resend(&mut self, now: Instant) {
        self.last_sent = now;
        self.retries += 1;
    }
}
