use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

pub enum Event {
    Client(Vec<u8>, SocketAddr),
    Retry,
}

/// How often we **check** a retry condition
const RETRY_CHECK_INTV: Duration = Duration::from_millis(20);

/// Spawns the receive and retry interval threads, returning a channel that
/// yields [Event] for the main thread to consume
pub fn spawn(skt: UdpSocket) -> Receiver<Event> {
    let (tx, rx) = mpsc::channel::<Event>();

    // Thread 1: forwards every received packet
    let tx_recv = tx.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];

        loop {
            let Ok((buf_len, skt_src)) = skt.recv_from(&mut buf) else {
                continue;
            };

            if tx_recv
                .send(Event::Client(buf[..buf_len].to_vec(), skt_src))
                .is_err()
            {
                break; // Main thread gone, shut down
            }
        }
    });

    // Thread 2: periodic nudge to check the pending map
    thread::spawn(move || {
        loop {
            thread::sleep(RETRY_CHECK_INTV);
            if tx.send(Event::Retry).is_err() {
                break;
            }
        }
    });

    rx
}
