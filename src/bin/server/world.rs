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
        util::{Hsv, rand_disk_pt_xz},
    },
};
use bitp::{
    bits::{BitReader, BitWriter},
    pkt::FieldName,
    qua,
};
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

    /// Spawns player in the simulation
    /// Returns a reliable buffer along with its id containing essential spawn
    /// information (location and color)
    pub fn spawn_player(
        &mut self,
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let ((spawn_x, spawn_z), spawn_pt_codec) = (
            rand_disk_pt_xz(10.0),
            qua::loc(self.codec.schema()), //
        );
        let (hsv, hsv_codec) = (
            Hsv::rand(),
            qua::hsv(self.codec.schema()), //
        );
        let (enc_x, enc_z) = (
            spawn_pt_codec.x.encode(spawn_x),
            spawn_pt_codec.z.encode(spawn_z), //
        );

        let fields = [
            (FieldName::LocationX, enc_x),
            (FieldName::LocationZ, enc_z),
            (FieldName::ColorH, hsv_codec.h.encode(hsv.h)),
            (FieldName::ColorS, hsv_codec.s.encode(hsv.s)),
            (FieldName::ColorV, hsv_codec.v.encode(hsv.v)),
        ];

        self.players.spawn(
            rel_target.sid,
            spawn_pt_codec.x.decode(enc_x),
            spawn_pt_codec.z.decode(enc_z),
            Hsv::new(hsv.h, hsv.s, hsv.v),
        );

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
        self.pker
            .players_snapshot_pkt(self.tick_ct, &self.players)
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
