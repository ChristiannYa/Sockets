use crate::manager::Manager;

pub fn tick(manager: &mut Manager) {
    let tick_ct = manager.world.tick();
    if !manager.world.ss_due() {
        return;
    };
    if let Some(pkt) = manager.world.pker().players_ss_pkt(tick_ct) {
        manager.net.broadcast(crate::net::Broadcast {
            addrs: manager.sess.addrs(),
            skt_src: None,
            buf: &pkt,
        });
    }
}
