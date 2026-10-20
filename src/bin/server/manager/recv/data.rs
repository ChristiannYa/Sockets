use crate::{codec::Rel, manager::Manager, net::Broadcast, world::RelTarget};
use std::{net::SocketAddr, time::Instant};
use tap::{Pipe, Tap};

pub fn recv(manager: &mut Manager, skt_src: SocketAddr, buf: &[u8]) {
    let Some(buf) = buf.get(2..) else {
        return;
    };

    if !manager.codec.schema().mask_fits(buf) {
        return;
    }

    // Capture client newness before potential registration
    let is_new_cli = manager.sess.is_new_cli(&skt_src);
    let sid = manager.sess.id(&skt_src);

    if is_new_cli {
        spawn_new_cli(manager, sid, &skt_src);
    }

    let mut reader = manager.codec.reader(buf);
    let (inid, outid) = manager.codec.pkt_ioids(&mut reader);

    if let Some(inid) = inid {
        manager
            .net
            .send_to(&bitp::rel::ack::encode(inid), skt_src);
        if is_dup(manager, inid, &skt_src) {
            return;
        }
    }

    out(manager, sid, outid, &mut reader, &skt_src);
}

fn spawn_new_cli(manager: &mut Manager, sid: u32, skt_src: &SocketAddr) {
    let mut rel_targ =
        RelTarget { sid, skt_src, pending_pkts: manager.pending_pkts };

    manager
        .world
        .pker()
        .welcome_pkt(&mut rel_targ)
        .tap(|pkt| manager.net.send_to(pkt, *skt_src));

    let (spawn_buf, sb_id) = manager.world.spawn_player(&mut rel_targ);
    manager.net.send_to(&spawn_buf, *skt_src);
    manager.net.broadcast_rel(
        Broadcast {
            addrs: manager.sess.addrs(),
            skt_src: Some(skt_src),
            buf: &spawn_buf,
        },
        sb_id,
        manager.pending_pkts,
    );

    if let Some(sync_buf) = manager.world.pker().meta_pkt(sid) {
        manager.net.send_to(&sync_buf, *skt_src);
    }
}

/// Returns `true` if the packet is a duplicate that the caller should drop.
fn is_dup(manager: &mut Manager, inid: u8, skt_src: &SocketAddr) -> bool {
    if manager.addrs_seen_pkt_ids.is_seen(skt_src, inid) {
        return true;
    }

    manager
        .addrs_seen_pkt_ids
        .mark_seen(skt_src, inid, Instant::now());

    false
}

fn out(
    manager: &mut Manager,
    sid: u32,
    outid: Option<u8>,
    reader: &mut bitp::bits::BitReader,
    skt_src: &SocketAddr,
) {
    let mut writer = manager.codec.writer();
    manager.codec.seed_pkt(
        &mut writer,
        sid,
        outid.map_or_else(
            || Rel::Seq(manager.sess.next_seq(skt_src)),
            Rel::Id,
        ),
    );

    let schema_pkt =
        manager
            .world
            .pker()
            .set_in_schema_pkt(sid, reader, &mut writer);

    Broadcast {
        addrs: manager.sess.addrs(),
        skt_src: Some(skt_src),
        buf: &schema_pkt,
    }
    .pipe(|args| match outid {
        Some(outid) => {
            manager
                .net
                .broadcast_rel(args, outid, manager.pending_pkts)
        }
        None => manager.net.broadcast(args),
    });
}
