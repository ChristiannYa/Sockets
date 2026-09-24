use std::{collections::HashMap, time::Instant};

use godot::prelude::{Array, Gd, GodotClass, PackedByteArray, godot_api};

use crate::rel::retx::{PendingPacket, RetryAction};

type PendingPackets = HashMap<u8, PendingPacket>;

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct Pending {
    pkts: PendingPackets,
    next_id: u8,
}

#[godot_api]
impl Pending {
    #[func]
    fn create() -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            pkts: PendingPackets::new(),
            next_id: 0,
        })
    }

    #[func]
    fn next_id(&mut self) -> u8 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    #[func]
    fn track(&mut self, id: u8, bytes: PackedByteArray) {
        self.pkts
            .insert(id, PendingPacket::new(id, bytes.to_vec(), Instant::now()));
    }

    #[func]
    fn ack(&mut self, id: u8) -> bool {
        self.pkts.remove(&id).is_some()
    }

    /// Sweeps every pending entry: due-and-under-limit entries are bumped and
    /// returned for resending.
    ///
    /// Returns the raw bytes of every packet that should be resent now.
    /// Caller is responsible for actually sending them.
    #[func]
    fn due_retry(&mut self) -> Array<PackedByteArray> {
        let now = Instant::now();
        let mut resend = Array::new();
        let mut giveups = Vec::<u8>::new();

        for (id, pkt) in self.pkts.iter_mut() {
            match pkt.retry_action(now) {
                RetryAction::Wait => {}
                RetryAction::Resend => {
                    resend.push(&PackedByteArray::from(pkt.bytes.as_slice()));
                    pkt.on_resend(now);
                }
                RetryAction::GiveUp => giveups.push(*id),
            }
        }

        for id in giveups {
            self.pkts.remove(&id);
        }

        resend
    }
}
