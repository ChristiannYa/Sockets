use crate::manager::Manager;

pub fn tick(manager: &mut Manager) {
    manager.world.tick();

    let tick_ct = manager.world.tick_ct();
    if let Some(pkt) = manager.world.pker().players_snapshot_pkt(tick_ct) {
        manager.net.broadcast(crate::net::Broadcast {
            addrs: manager.sess.addrs(),
            skt_src: None,
            buf: &pkt,
        });
    }
}
