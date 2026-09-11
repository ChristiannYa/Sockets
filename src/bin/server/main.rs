mod net;
mod packer;
mod session;
mod world;

use crate::{net::Net, packer::Packer, session::Session, world::state::State};

fn main() {
    let net = Net::build("0.0.0.0:34254");
    let packer = Packer::build();
    let mut sess = Session::new();
    let mut state = State::new();

    let mut buf = [0; 1024];

    loop {
        let (skt_src_buflen, skt_src) = net.recv_from(&mut buf);

        // Bail early on packets too short to even hold the mask bytes the
        // schema expects.
        // Protects `BitReader::new()`'s `split_at()` from panicking and
        // taking the whole server down over one client's bad packet.
        if skt_src_buflen < packer.schema().fields.len().div_ceil(8) {
            continue;
        }

        // Capture client newness before potential registration
        let is_new_cli = sess.is_new_cli(&skt_src);
        let sid = sess.sid_or_reg(&skt_src);

        if is_new_cli {
            net.send_to(&packer.welcome(sid), skt_src);

            if !state.clis().is_empty() {
                net.send_to(&packer.sync(state.clis(), sid), skt_src);
            }
        }

        let buf = &buf[..skt_src_buflen];
        let seq = sess.next_seq(&skt_src);

        let (decoded, broadcast_buf) = packer.process(buf, sid, seq);
        for (field_name, val) in decoded {
            println!("@{skt_src} #{sid}: {field_name:?}={val}");
            state.save_cli(&field_name, sid, val);
        }

        net.broadcast(sess.addrs(), &skt_src, &broadcast_buf);
    }
}
