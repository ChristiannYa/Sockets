use crate::{codec::Rel, manager::Manager, net::Broadcast, world::RelTarget};
use std::{net::SocketAddr, time::Instant};
use tap::Pipe;

pub struct DataPkt<'d, 'c, 'w> {
    manager: &'d mut Manager<'c, 'w>,
    skt_src: SocketAddr,
    buf: &'d [u8],
}

impl<'d, 'c, 'w> DataPkt<'d, 'c, 'w> {
    pub fn new(manager: &'d mut Manager<'c, 'w>, skt_src: SocketAddr, buf: &'d [u8]) -> Self {
        DataPkt {
            manager,
            skt_src,
            buf,
        }
    }

    pub fn recv(&mut self) {
        if !self.manager.codec.schema().mask_fits(self.buf) {
            return;
        }

        // Capture client newness before potential registration
        let is_new_cli = self.manager.sess.is_new_cli(&self.skt_src);
        let sid = self.manager.sess.sid(&self.skt_src);

        if is_new_cli {
            self.spawn_new_cli(sid);
        }

        let mut reader = self.manager.codec.reader(self.buf);
        let (inid, outid) = self.manager.codec.pkt_ioids(&mut reader);

        if let Some(inid) = inid {
            self.manager
                .net
                .send_to(&bitp::rel::ack::encode(inid), self.skt_src);
            if self.is_dup(inid) {
                return;
            }
        }

        self.ship(sid, outid, &mut reader);
    }

    fn spawn_new_cli(&mut self, sid: u32) {
        let mut rel_targ = RelTarget {
            sid,
            skt_src: &self.skt_src,
            pending_pkts: self.manager.pending_pkts,
        };

        let welcome_buf = self.manager.world.welcome_buf(&mut rel_targ);
        self.manager.net.send_to(&welcome_buf, self.skt_src);

        // Capture world state before spawning player and saving its state in
        // the world/session
        let is_world_stateful = self.manager.world.has_state();

        let (spawn_buf, sb_id) = self.manager.world.spawn_buf(&mut rel_targ);
        self.manager.net.send_to(&spawn_buf, self.skt_src);
        self.manager.net.broadcast_rel(
            Broadcast {
                addrs: self.manager.sess.addrs(),
                skt_src: &self.skt_src,
                buf: &spawn_buf,
            },
            sb_id,
            self.manager.pending_pkts,
        );

        if is_world_stateful {
            self.manager
                .net
                .send_to(&self.manager.world.sync_buf(sid), self.skt_src);
        }
    }

    /// Returns `true` if the packet is a duplicate that the caller should drop.
    fn is_dup(&mut self, inid: u8) -> bool {
        if self.manager.addrs_seen_pkt_ids.is_seen(&self.skt_src, inid) {
            return true;
        }

        self.manager
            .addrs_seen_pkt_ids
            .mark_seen(&self.skt_src, inid, Instant::now());

        false
    }

    fn ship(&mut self, sid: u32, outid: Option<u8>, reader: &mut bitp::bits::BitReader) {
        let mut writer = self.manager.codec.writer();
        self.manager.codec.seed_pkt(
            &mut writer,
            sid,
            outid.map_or_else(
                || Rel::Seq(self.manager.sess.next_seq(&self.skt_src)),
                Rel::Id,
            ),
        );

        let world_buf = self.manager.world.process(sid, reader, &mut writer);

        Broadcast {
            addrs: self.manager.sess.addrs(),
            skt_src: &self.skt_src,
            buf: &world_buf,
        }
        .pipe(|args| match outid {
            Some(outid) => self
                .manager
                .net
                .broadcast_rel(args, outid, self.manager.pending_pkts),
            None => self.manager.net.broadcast(args),
        });
    }
}
