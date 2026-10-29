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

mod dbg;
mod players;
mod utils;

use crate::{bits::PacketSchema, pkt::FIELD_LENGTHS, sim::consts::SS_INTV_S};
use dbg::StateDebug;
use godot::prelude::{Gd, GodotClass, PackedByteArray, godot_api};
use players::PlayerStates;
use tap::Pipe;
use utils::{Clock, LocalSim};

/// `[PacketKind::Ss, tick_lo, tick_hi]`, followed by the records
const SS_HEADER_LEN: usize = 3;

/// Extra lag added on purpose so jitter does not starve interpolation,
/// causing player freezes.
///
/// Time-based: jitter in ms doesn't depend on the snapshot rate. Needs to
/// stay >= ~2 snapshot intervals, one is for the regular interval and the
/// second is so that losing a single snapshot doesn't starve the
/// interpolation.
///
/// Drawing at `T = now - delay` needs the snapshot right AFTER `T` to have
/// arrived by `now`. If it hasn't, there's nothing to interpolate toward
/// ("starved") and the player freezes at its last pose:
///
/// ```text
/// S5 arrived:   S4 ------ S5 ------ S6
///                  ^T     ^ interpolate S4 -> S5, smooth
///
/// S5 lost:      S4 ------ S5 ------ S6
///                  ^T     X         ^ the next "after" snapshot is 2
///                                     intervals past T and must arrive by
///                                     `now`, else: starved, freeze
/// ```
const RENDER_DELAY_S: f64 = 0.100;
const _: () = assert!(
    RENDER_DELAY_S >= 2.0 * SS_INTV_S,
    "render delay must be at least 2 snapshot intervals"
);

/// Gap between snapshot arrivals. If longer than this gets logged.
/// 3 snapshot intervals, so about 2 missing snapshots: 50ms at 60hz, 150ms at
/// 20hz
const GAP_WARN_S: f64 = 3.0 * SS_INTV_S;

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpState {
    players: Gd<PlayerStates>,
    locsim: Gd<LocalSim>,
    clock: Clock,
    dbg: StateDebug,
}

#[godot_api]
impl UdpState {
    #[func]
    fn create() -> Gd<Self> {
        let schema = PacketSchema::build(&FIELD_LENGTHS).unwrap();
        Gd::from_init_fn(|_| Self {
            players: Gd::from_object(PlayerStates::new(schema)),
            locsim: Gd::from_object(LocalSim::new()),
            clock: Clock::new(),
            dbg: StateDebug::default(),
        })
    }

    #[func]
    fn players(&self) -> Gd<PlayerStates> { self.players.clone() }

    #[func]
    fn locsim(&self) -> Gd<LocalSim> { self.locsim.clone() }

    /// Takes a whole snapshot packet and updates the players from it.
    ///
    /// `arrival_s` is when it arrived. Returns `false` when the packet was
    /// dropped: too short, or not newer than the last snapshot (a duplicate
    /// or a reordered packet).
    #[func]
    fn ingest_ss(&mut self, buf: PackedByteArray, arrival_s: f64) -> bool {
        let (Some([_, lo, hi]), Some(records)) = buf
            .as_slice()
            .pipe(|b| (b.get(..SS_HEADER_LEN), b.get(SS_HEADER_LEN..)))
        else {
            return false;
        };

        let Some(ss_s) = self
            .clock
            .accept(u16::from_le_bytes([*lo, *hi]), arrival_s)
        else {
            return false;
        };

        self.dbg.ss_wtch.track_gap(arrival_s, ss_s);

        {
            let server = {
                let mut players = self.players.bind_mut();
                players.apply_ss(records, ss_s);
                players.take_my_server_state()
            };

            if let Some(server) = server {
                self.locsim.bind_mut().push_server_state(server);
            }
        }

        true
    }

    /// The moment in the past that should be on screen right now
    #[func]
    fn render_t(&self, now_s: f64) -> f64 { now_s - RENDER_DELAY_S }
}
