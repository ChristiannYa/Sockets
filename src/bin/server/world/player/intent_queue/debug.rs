use bitp::logt;

/// Pops in a row without a queued intent before it gets a log line
const DRY_POPS_WARN: u32 = 3;

pub struct PlayerIntentQueueDbg {
    /// Ticks in a row where the queue was empty
    dry_pops: u32,
}

impl PlayerIntentQueueDbg {
    pub fn new() -> PlayerIntentQueueDbg {
        PlayerIntentQueueDbg { dry_pops: 0 }
    }

    pub fn on_queue_found(&mut self) {
        if self.dry_pops >= DRY_POPS_WARN {
            logt!("intents flowing again after {} dry pops", self.dry_pops);
        }
        self.dry_pops = 0;
    }

    pub fn on_queue_dry(&mut self) {
        self.dry_pops += 1;
        if self.dry_pops == DRY_POPS_WARN {
            logt!(
                "intent queue ran dry for {} pops, repeating last input",
                DRY_POPS_WARN
            );
        }
    }
}
