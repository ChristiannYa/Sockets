mod pker;
pub mod player;
mod sim;

use crate::{
    codec::Codec,
    rel::retx::PendingPackets,
    world::{
        pker::WorldPacker,
        player::{Player, Players},
    },
};
use bitp::sim::consts::SS_EVERY_TICKS;
use std::net::SocketAddr;

/// Who to send to and where to track it
pub struct RelTarget<'r> {
    pub sid: u32,
    pub skt_src: &'r SocketAddr,
    pub pending_pkts: &'r mut PendingPackets,
}

pub struct World<'w> {
    codec: &'w Codec,
    tick_ct: u16,
    ticks_since_ss: u8,
    players: Players,
}

impl<'w> World<'w> {
    pub fn new(codec: &'w Codec) -> Self {
        World {
            codec,
            tick_ct: 0,
            ticks_since_ss: 0,
            players: Players::default(),
        }
    }

    /// Advances the simulation by 1 tick and returns the increased tick count
    pub fn tick(&mut self) -> u16 {
        self.tick_ct = self.tick_ct.wrapping_add(1);
        self.players.sim_tick();
        self.tick_ct
    }

    pub fn ss_due(&mut self) -> bool {
        self.ticks_since_ss += 1;
        if self.ticks_since_ss < SS_EVERY_TICKS {
            false
        } else {
            self.ticks_since_ss = 0;
            true
        }
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
        self.pker().rel_pkting(&fields, rel_target)
    }

    pub fn pker(&self) -> WorldPacker<'_> {
        WorldPacker::new(self.codec, &self.players)
    }
}
