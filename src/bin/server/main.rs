mod codec;
mod events;
mod net;
mod rel;
mod session;
mod world;

use crate::{
    codec::Codec,
    events::Event,
    net::Net,
    rel::{dedup::AddrsSeenPacketIds, retx::PendingPackets},
    session::Session,
    world::World,
};
use bitp::rel::{
    ack::PacketType,
    retx::{PendingPacket, RetryAction},
};
use std::{net::SocketAddr, time::Instant};

struct Ctx<'c, 'w> {
    net: &'c Net,
    sess: &'c mut Session,
    codec: &'c Codec,
    world: &'c mut World<'w>,
    pending_pkts: &'c mut PendingPackets,
    addrs_seen_pkt_ids: &'c mut AddrsSeenPacketIds,
}

fn main() {
    let net = Net::build("0.0.0.0:34254");
    let evs_rx = events::spawn(net.try_clone_skt());

    let mut sess = Session::new();
    let codec = Codec::new();
    let mut world = World::new(&codec);

    let mut pending_pkts = PendingPackets::new();
    let mut addrs_seen_ids = AddrsSeenPacketIds::new();

    for ev in evs_rx {
        let mut ctx = Ctx {
            net: &net,
            sess: &mut sess,
            codec: &codec,
            world: &mut world,
            pending_pkts: &mut pending_pkts,
            addrs_seen_pkt_ids: &mut addrs_seen_ids,
        };

        match ev {
            Event::Retry => handle_retry(&mut ctx),
            Event::Client(buf, skt_src) => handle_cli_pkt(&mut ctx, &buf, skt_src),
        }
    }
}

fn handle_retry(ctx: &mut Ctx) {
    let mut giveups = Vec::<(u8, SocketAddr)>::new();
    let now = Instant::now();

    for ((id, addr), pkt) in ctx.pending_pkts.iter_mut() {
        match pkt.retry_action(now) {
            RetryAction::Wait => {}
            RetryAction::Resend => {
                ctx.net.send_to(&pkt.bytes, *addr);
                pkt.on_resend(now);
            }
            RetryAction::GiveUp => giveups.push((*id, *addr)),
        }
    }

    for key in giveups {
        ctx.pending_pkts.remove(&key);
    }

    ctx.addrs_seen_pkt_ids.sweep_all(now);
}

fn handle_cli_pkt(ctx: &mut Ctx, buf: &[u8], skt_src: SocketAddr) {
    match bitp::rel::ack::decode(buf) {
        Some(id) => {
            crate::rel::ack::handle_ack(ctx.pending_pkts, id, &skt_src);
        }
        None if buf.first() == Some(&(PacketType::Data as u8)) => {
            // Strip header bytes because they are not defined in the schema
            if let Some(buf) = buf.get(2..) {
                handle_data_pkt(ctx, buf, skt_src);
            };
        }
        None => {}
    }
}

fn handle_data_pkt(ctx: &mut Ctx, buf: &[u8], skt_src: SocketAddr) {
    // Bail early on packets too short to even hold the mask bytes the schema
    // expects.
    // Protects `BitReader::new()`'s `split_at()` from panicking and
    // taking the whole server down over one client's bad packet.
    if buf.len() < ctx.codec.schema().fields.len().div_ceil(8) {
        return;
    }

    // Capture client newness before potential registration
    let is_new_cli = ctx.sess.is_new_cli(&skt_src);

    let sid = ctx.sess.sid(&skt_src);

    if is_new_cli {
        ctx.net.send_to(&ctx.world.welcome_buf(sid), skt_src);

        // Capture world state before spawning player and saving its state in
        // the world/session
        let is_world_stateful = ctx.world.has_state();

        let buf = ctx.world.player_spawn_buf(sid);
        ctx.net.send_to(&buf, skt_src);
        ctx.net.broadcast(ctx.sess.addrs(), &skt_src, &buf);

        if is_world_stateful {
            ctx.net.send_to(&ctx.world.sync_buf(sid), skt_src);
        }
    }

    let (pkt_io_ids, mut reader, mut writer) =
        ctx.codec
            .preprocess_pkt(buf, sid, ctx.sess.next_seq(&skt_src));

    let buf = ctx.world.process(sid, &mut reader, &mut writer);

    if let Some((pkt_id_in, _)) = pkt_io_ids {
        ctx.net.send_to(&bitp::rel::ack::encode(pkt_id_in), skt_src);

        if ctx.addrs_seen_pkt_ids.is_seen(&skt_src, pkt_id_in) {
            return;
        }

        ctx.addrs_seen_pkt_ids
            .mark_seen(&skt_src, pkt_id_in, Instant::now());
    }

    match pkt_io_ids {
        Some((_, pkt_id_out)) => {
            let now = Instant::now();
            for addr in ctx.sess.addrs() {
                if *addr == skt_src {
                    continue;
                }
                ctx.net.send_to(&buf, *addr);
                ctx.pending_pkts.insert(
                    (pkt_id_out, *addr),
                    PendingPacket::new(pkt_id_out, buf.clone(), now),
                );
            }
        }
        None => ctx.net.broadcast(ctx.sess.addrs(), &skt_src, &buf),
    }
}
