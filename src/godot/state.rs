//! What Godot calls into.
//!
//! Each module here (except `utils`) holds one kind of game state, and is
//! built from the same four layers, bottom to top.
//!
//! ## (1) Groups
//!
//! One small `Raw*` module per kind of data. The raw type holds fields that
//! have not been decoded yet, and it is used in the delta (layer 2) and in
//! the state (layer 3).
//!
//! It can store a field (`set()`), update itself with the fields of a record
//! that just arrived (`merge()`), and turn its fields into real values once
//! all of them are known (`decode()`).
//!
//! ---
//! A merge keeps the old value of any field the record doesn't carry.
//!
//! ```text
//! stored: x=10, y=10, z=5
//! new:    x=12, y=None, z=None
//! result: x=12, y=10, z=5
//! ```
//!
//! ## (2) The delta
//!
//! Reads one record from the wire in a single pass.
//! It stores what that one record carried, nothing else.
//!
//! ## (3) The state
//!
//! What is known about one entity: the groups' data, brought up to date by
//! applying each delta in turn. Every record carries only part of the
//! picture, so this is where the parts add up, and it always holds the
//! latest known value of everything.
//!
//! ## (4) The module
//!
//! Keeps the state of every entity, by session id, and is the only layer
//! Godot sees.
//!
//! ```text
//! record ──> delta ──┬─ pose  ──┐
//!  (bytes)           ├─ color ──┼─> state of one player ──┐
//!                    └─ seq   ──┘                         ├─> mod ──> Godot
//!                          state of every other player ───┘
//! ```

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
