use crate::rel::retx::PendingPackets;
use bitp::rel::retx::PendingPacket;
use std::{
    net::{SocketAddr, UdpSocket},
    time::Instant,
};

pub struct Net {
    skt: UdpSocket,
}

pub struct Broadcast<'a, I>
where
    I: Iterator<Item = &'a SocketAddr>,
{
    pub addrs: I,
    pub skt_src: Option<&'a SocketAddr>,
    pub buf: &'a [u8],
}

impl Net {
    /// Panics if UDP socket creation from the given address fails
    pub fn build(addr: &str) -> Self {
        let skt = UdpSocket::bind(addr)
            .unwrap_or_else(|e| panic!("Couldn't bind {addr}: {e}"));
        Net { skt }
    }

    pub fn try_clone_skt(&self) -> UdpSocket {
        self.skt
            .try_clone()
            .expect("Couldn't clone socket")
    }

    /// Sends data on the socket to the given address
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) {
        self.skt.send_to(buf, addr).ok();
    }

    /// Sends data to all known addresses excluding the socket source
    pub fn broadcast<'a, I>(&self, args: Broadcast<'a, I>)
    where
        I: Iterator<Item = &'a SocketAddr>,
    {
        let buf = args.buf;
        self.on_broadcast(args, |addr| {
            self.send_to(buf, *addr);
        });
    }

    /// Sends `buf` to every address except `skt_src`, tracking a
    /// `PendingPacket` per recipient under `(id, addr)` so each can be
    /// retried independently until acked.
    pub fn broadcast_rel<'a, I>(
        &self,
        args: Broadcast<'a, I>,
        pkt_id: u8,
        pending_pkts: &mut PendingPackets,
    ) where
        I: Iterator<Item = &'a SocketAddr>,
    {
        let now = Instant::now();
        let buf = args.buf;

        self.on_broadcast(args, |addr| {
            self.send_to(buf, *addr);
            pending_pkts.insert(
                (pkt_id, *addr),
                PendingPacket::new(pkt_id, buf.to_vec(), now),
            );
        });
    }

    fn on_broadcast<'f, I, H>(&self, args: Broadcast<'f, I>, mut handle: H)
    where
        I: Iterator<Item = &'f SocketAddr>,
        H: FnMut(&SocketAddr),
    {
        if let Some(skt_src) = args.skt_src {
            for addr in args.addrs {
                if *addr != *skt_src {
                    handle(addr);
                }
            }
        } else {
            for addr in args.addrs {
                handle(addr);
            }
        }
    }
}
