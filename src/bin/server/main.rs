mod net;
mod session;
mod world;

use crate::{net::Net, session::Session, world::World};

fn main() {
    let net = Net::build("0.0.0.0:34254");
    let mut sess = Session::new();
    let mut world = World::new();

    let mut buf = [0; 1024];

    loop {
        let (skt_src_buflen, skt_src) = net.recv_from(&mut buf);

        // Bail early on packets too short to even hold the mask bytes the
        // schema expects.
        // Protects `BitReader::new()`'s `split_at()` from panicking and
        // taking the whole server down over one client's bad packet.
        if skt_src_buflen < world.schema_min_len() {
            continue;
        }

        // Capture client newness before potential registration
        let is_new_cli = sess.is_new_cli(&skt_src);

        let sid = sess.sid_or_reg(&skt_src);

        if is_new_cli {
            net.send_to(&world.welcome_buf(sid), skt_src);

            let is_world_empty = world.is_empty();

            let buf = world.player_spawn_buf(sid);
            net.send_to(&buf, skt_src);
            net.broadcast(sess.addrs(), &skt_src, &buf);

            if !is_world_empty {
                net.send_to(&world.sync_buf(sid), skt_src);
            }
        }

        let seq = sess.next_seq(&skt_src);
        let buf = &buf[..skt_src_buflen];
        let broadcast_buf = world.process(buf, sid, seq);
        net.broadcast(sess.addrs(), &skt_src, &broadcast_buf);
    }
}
