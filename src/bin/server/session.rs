use std::{collections::HashMap, net::SocketAddr};

pub struct Session {
    clis: HashMap<SocketAddr, u32>,
    seqs: HashMap<SocketAddr, u8>,
    next: u32,
}

impl Session {
    pub fn new() -> Self {
        Session {
            clis: HashMap::<SocketAddr, u32>::new(),
            seqs: HashMap::<SocketAddr, u8>::new(),
            next: 0,
        }
    }

    pub fn sid(&mut self, skt_src: &SocketAddr) -> u32 {
        *self.clis.entry(*skt_src).or_insert_with(|| {
            let next = self.next;
            self.next += 1;
            next
        })
    }

    /// Advances and returns the sequence number for the client's next packet.
    pub fn next_seq(&mut self, skt_src: &SocketAddr) -> u8 {
        let seq = self.seqs.entry(*skt_src).or_insert(0);
        *seq = seq.wrapping_add(1);
        *seq
    }

    pub fn is_new_cli(&self, skt_src: &SocketAddr) -> bool {
        !self.clis.contains_key(skt_src)
    }

    pub fn addrs(&self) -> impl Iterator<Item = &SocketAddr> {
        self.clis.keys()
    }
}
