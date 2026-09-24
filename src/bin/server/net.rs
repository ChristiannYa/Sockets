use std::net::{SocketAddr, UdpSocket};

pub struct Net {
    skt: UdpSocket,
}

impl Net {
    /// Panics if UDP socket creation from the given address fails
    pub fn build(addr: &str) -> Self {
        let skt = UdpSocket::bind(addr).expect("Couldn't bind");
        Net { skt }
    }

    pub fn try_clone_skt(&self) -> UdpSocket {
        self.skt.try_clone().expect("Couldn't clone socket")
    }

    /// Sends data on the socket to the given address
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) {
        self.skt.send_to(buf, addr).ok();
    }

    /// Sends data to all known addresses excluding the socket source
    pub fn broadcast<'a>(
        &self,
        addrs: impl Iterator<Item = &'a SocketAddr>,
        skt_src: &SocketAddr,
        buf: &[u8],
    ) {
        for addr in addrs {
            if *addr != *skt_src {
                self.skt.send_to(buf, addr).ok();
            }
        }
    }
}
