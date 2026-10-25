use bitp::{logt, sim::consts::SIM_HZ};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

pub enum Event {
    Client(Vec<u8>, SocketAddr),
    Retry,
    SimTick,
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Event::Client(..) => "client",
            Event::Retry => "retry",
            Event::SimTick => "sim_tick",
        }
    }
}

const RETRY_CHECK_INTV: Duration = Duration::from_millis(20);

/// [`Duration::from_micros`] works, but since we're deriving from [SIM_HZ],
/// integer division throws away the remainder, and the unit decides how much
/// it is thrown away:
///
/// - *Microseconds*: `1_000_000 / 60` gives 16,666 µs and drops 0.67 µs per
///   tick. Over an hour adds up to ~0.14s of drift against an exact 60hz
/// - *Nanoseconds*: `1_000_000_000 / 60` gives 16,666,666 ns and drops 0.67s
///   per tick. Over an hour adds up to ~0.14ms of drift against an exact 60hz
///
/// Both small for gameplay, so micros could be used, but nanos are more
/// accurate for free.
const SIM_TICK_INTV: Duration =
    Duration::from_nanos(1_000_000_000 / SIM_HZ as u64);

/// Spawns the receive, sim tick, and retry interval threads, returning a
/// channel that yields [Event] for the main thread to consume
pub fn spawn(skt: UdpSocket) -> Receiver<(Instant, Event)> {
    let (tx, rx) = mpsc::channel::<(Instant, Event)>();

    // Forward every received packet
    let tx_recv = tx.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];

        loop {
            let Ok((buf_len, skt_src)) = skt.recv_from(&mut buf) else {
                continue;
            };

            if tx_recv
                .send((
                    Instant::now(),
                    Event::Client(buf[..buf_len].to_vec(), skt_src),
                ))
                .is_err()
            {
                break; // Main thread gone, shut down
            }
        }
    });

    // Fixed-rate simulation tick
    let tx_tick = tx.clone();
    thread::spawn(move || {
        let mut next: Instant = Instant::now() + SIM_TICK_INTV;
        loop {
            thread::sleep(next.saturating_duration_since(Instant::now()));

            let late = Instant::now().saturating_duration_since(next);
            if late.as_millis() >= 2 {
                logt!("tick thread late by {late:?}");
            }

            if tx_tick.send((Instant::now(), Event::SimTick)).is_err() {
                break;
            }

            // Computing a `next` deadline instad of calling
            // `sleep(SIM_TICK_INTV)` each loop stops the rate from drifting
            // which can be caused by `thread::sleep` not guaranteeing the
            // exact sleep duration
            next += SIM_TICK_INTV;
        }
    });

    // Periodic nudge to check the pending map
    thread::spawn(move || {
        loop {
            thread::sleep(RETRY_CHECK_INTV);

            if tx.send((Instant::now(), Event::Retry)).is_err() {
                break;
            }
        }
    });

    rx
}
