mod players;
mod utils;

use crate::{bits::PacketSchema, pkt::FIELD_LENGTHS};
use godot::prelude::{Gd, GodotClass, PackedByteArray, godot_api};
use players::PlayerStates;
use utils::Clock;

/// `[PacketKind::Snapshot, tick_lo, tick_hi]`, followed by the records
const SNAPSHOT_HEADER_LEN: usize = 3;

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpState {
    players: Gd<PlayerStates>,
    clock: Clock,
}

#[godot_api]
impl UdpState {
    #[func]
    fn create(history_cap: i64) -> Gd<Self> {
        let clock = Clock::new();

        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        let cap = history_cap.max(2) as usize;
        let players = Gd::from_object(PlayerStates::new(schema, cap));

        Gd::from_init_fn(|_| Self { players, clock })
    }

    #[func]
    fn players(&self) -> Gd<PlayerStates> { self.players.clone() }

    /// Takes a whole snapshot packet and updates the players from it.
    ///
    /// `arrival_s` is when it arrived. Returns `false` when the packet was
    /// dropped: too short, or not newer than the last snapshot (a duplicate or
    /// a reordered packet).
    #[func]
    fn ingest_ss(&mut self, buf: PackedByteArray, arrival_s: f64) -> bool {
        let bytes = buf.as_slice();
        let (Some([_, lo, hi]), Some(records)) = (
            bytes.get(..SNAPSHOT_HEADER_LEN),
            bytes.get(SNAPSHOT_HEADER_LEN..),
        ) else {
            return false;
        };

        let tick = u16::from_le_bytes([*lo, *hi]);
        let Some(snapshot_s) = self.clock.accept(tick, arrival_s) else {
            return false;
        };

        self.players
            .bind_mut()
            .apply_snapshot(records, snapshot_s);
        true
    }
}
