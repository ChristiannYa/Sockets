mod pker;
pub mod player;
mod sim;
mod util;

use crate::{
    codec::Codec,
    rel::retx::PendingPackets,
    world::{
        pker::WorldPacker,
        player::{Player, Players},
    },
};
use bitp::bits::{BitReader, BitWriter};
use std::net::SocketAddr;

/// Who to send to and where to track it
pub struct RelTarget<'r> {
    pub sid: u32,
    pub skt_src: &'r SocketAddr,
    pub pending_pkts: &'r mut PendingPackets,
}

pub struct World<'w> {
    tick_ct: u16,
    pker: WorldPacker<'w>,
    codec: &'w Codec,
    players: Players,
}

impl<'w> World<'w> {
    pub fn new(codec: &'w Codec) -> Self {
        World {
            pker: WorldPacker::new(codec),
            codec,
            players: Players::default(),
            tick_ct: 0,
        }
    }

    /// Advances the simulation by 1 tick
    pub fn tick(&mut self) {
        self.tick_ct = self.tick_ct.wrapping_add(1);
        self.players.sim_tick();
    }

    pub fn player_mut_ref(&mut self, sid: u32) -> Option<&mut Player> {
        self.players.by_sid_mut_ref(sid)
    }

    /// Returns a reliable buffer along with its id containing essential spawn
    /// information
    pub fn spawn_player(
        &mut self,
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let (player, fields) = Player::spawn(self.codec.schema());
        self.players.insert_new(player, rel_target.sid);
        self.pker.rel_pkting(&fields, rel_target)
    }

    // --------------
    // Paker relays
    // --------------

    pub fn welcome_pkt(&self, rel_target: &mut RelTarget) -> Vec<u8> {
        self.pker.welcome_pkt(rel_target)
    }

    pub fn sync_pkt(&self, sid: u32) -> Option<Vec<u8>> {
        self.pker.sync_pkt(sid, &self.players)
    }

    pub fn players_snapshot_pkt(&self) -> Option<Vec<u8>> {
        self.pker.players_snapshot_pkt(self.tick_ct, &self.players)
    }

    pub fn schema_pkt(
        &mut self,
        sid: u32,
        reader: &mut BitReader,
        writer: &mut BitWriter,
    ) -> Vec<u8> {
        self.pker.schema_pkt(sid, reader, writer)
    }
}
