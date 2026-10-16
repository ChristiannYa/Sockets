use crate::manager::Manager;

pub fn tick(manager: &mut Manager) {
    manager.world.tick();
    if let Some(pkt) = manager.world.players_snapshot_pkt() {
        manager.net.broadcast(crate::net::Broadcast {
            addrs: manager.sess.addrs(),
            skt_src: None,
            buf: &pkt,
        });
    }
}
